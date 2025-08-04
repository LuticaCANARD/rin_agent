use bytes::Bytes;
use dashmap::DashMap;
use serenity::all::{ChannelId, Context, GuildId};
use songbird::input::{Input, RawAdapter};
use songbird::{Call, Songbird};
use std::io;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use std::task::{self, Poll};
use tokio::io::AsyncRead;
use tokio::sync::{mpsc, Mutex};

use crate::libs::logger::{LogLevel, LOGGER};

// --- 오디오 청크를 비동기 채널로부터 읽어오는 커스텀 리더 ---
// songbird가 오디오를 재생할 수 있도록 AsyncRead 트레이트를 구현합니다.
pub struct ChunkStream {
    rx: mpsc::Receiver<Bytes>,
}

impl AsyncRead for ChunkStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // 채널에서 다음 오디오 청크를 비동기적으로 수신 시도합니다.
        match self.rx.poll_recv(cx) {
            Poll::Ready(Some(chunk)) => {
                buf.put_slice(&chunk); // 버퍼에 데이터 복사
                Poll::Ready(Ok(()))
            }
            Poll::Ready(None) => Poll::Ready(Ok(())), // 채널이 닫힘 (스트림 끝)
            Poll::Pending => Poll::Pending,           // 데이터가 아직 없음
        }
    }
}

// --- 길드별 음성 상태를 저장하는 구조체 ---
// 음성 핸들러와 오디오 청크를 보낼 채널의 송신부를 가집니다.
pub struct GuildVoiceState {
    pub call: Arc<Mutex<Call>>,
    pub audio_sender: mpsc::Sender<Bytes>,
}

// --- 싱글턴 음성 매니저 ---
#[derive(Default)]
pub struct VoiceManager {
    // DashMap을 사용하여 여러 스레드에서 안전하게 길드별 상태를 관리합니다.
    guild_states: DashMap<GuildId, Arc<GuildVoiceState>>,
}

impl VoiceManager {
    /// 음성 채널에 참여하고, 오디오 파이프라인을 설정하며, 상태를 저장합니다.
    /// 이 함수는 오디오 재생을 위한 별도의 비동기 태스크를 생성합니다.
    pub async fn join(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Result<Arc<GuildVoiceState>, String> {
        // 이미 해당 길드의 음성 채널에 참여 중이라면, 기존 상태를 반환합니다.
        if let Some(state) = self.guild_states.get(&guild_id) {
            LOGGER.log(LogLevel::Info, &format!("[VoiceManager] Already in channel for guild {}", guild_id));
            return Ok(state.clone());
        }

        LOGGER.log(LogLevel::Info, &format!("[VoiceManager] Joining channel {} in guild {}", channel_id, guild_id));
        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in scope at initialization.")
            .clone();

        let call_lock = match manager.join(guild_id, channel_id).await {
            Ok(call) => call, // 성공 시 `Arc<Mutex<Call>>`을 얻습니다.
            Err(e) => {
                // 실패 시 에러를 포맷팅하여 반환합니다.
                return Err(format!("Failed to join channel: {:?}", e));
            }
        };

        // 오디오 청크를 전송할 채널을 생성합니다.
        let (audio_sender, audio_receiver) = mpsc::channel(100);

        // 길드 상태 객체를 생성합니다.
        let state = Arc::new(GuildVoiceState {
            call: call_lock.clone(),
            audio_sender,
        });

        // 이 길드의 음성 연결을 위한 전용 태스크를 생성합니다.
        // 이 태스크는 채널에서 오디오를 받아 재생하는 역할만 담당합니다.
        tokio::spawn(async move {
            let stream_reader = ChunkStream { rx: audio_receiver };

            // 오디오 포맷 정의: PCM s16le, 48kHz, 2채널 (스테레오)
            let pcm_type = songbird::input::codecs::RawCodec::PcmS16Le;
            let pcm_rate = 48_000;
            let pcm_channels = 2;

            let raw_adapter = RawAdapter::new(stream_reader, pcm_type, pcm_rate);
            let input = Input::from(raw_adapter);

            let mut call = call_lock.lock().await;
            call.play_source(input);
            
            LOGGER.log(LogLevel::Info, &format!("[VoiceManager] Playback task started for guild {}", guild_id));
        });

        // 생성된 상태를 DashMap에 저장하고 복사본을 반환합니다.
        self.guild_states.insert(guild_id, state.clone());
        Ok(state)
    }

    /// 음성 채널을 떠나고, 관련 상태와 태스크를 정리합니다.
    pub async fn leave(&self, ctx: &Context, guild_id: GuildId) -> Result<(), String> {
        LOGGER.log(LogLevel::Info, &format!("[VoiceManager] Leaving channel in guild {}", guild_id));
        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in scope at initialization.")
            .clone();
        
        if let Err(e) = manager.leave(guild_id).await {
            return Err(format!("Failed to leave channel: {:?}", e));
        }

        // DashMap에서 상태를 제거합니다.
        // audio_sender가 drop되면서 채널이 닫히고, 재생 태스크는 자동으로 종료됩니다.
        self.guild_states.remove(&guild_id);
        
        Ok(())
    }

    /// 특정 길드의 음성 채널로 오디오 청크를 보냅니다.
    pub async fn play(&self, guild_id: GuildId, chunk: Bytes) -> Result<(), String> {
        if let Some(state) = self.guild_states.get(&guild_id) {
            if let Err(e) = state.audio_sender.send(chunk).await {
                return Err(format!("Failed to send audio chunk: {:?}", e));
            }
            Ok(())
        } else {
            Err("Not in a voice channel in this guild.".to_string())
        }
    }
}
pub static VOICE_MANAGER: LazyLock<VoiceManager> = LazyLock::new(VoiceManager::default);