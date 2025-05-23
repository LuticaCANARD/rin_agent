use std::{collections::hash_map, env};
use std::sync::LazyLock;

use serde_json::json;
use serenity::all::User;
use sqlx::types::chrono;


use gemini_live_api::types::{
    GeminiGenerationConfig, 
    ThinkingConfig
};
use crate::{gemini::{types::{GeminiBotTools, GeminiChatChunk}, utils::generate_fns_to_gemini}, libs::logger::LOGGER};

pub const GEMINI_MODEL_PRO : &str = "gemini-2.5-pro-preview-03-25";
pub const GEMINI_MODEL_FLASH: &str = "gemini-2.5-flash-preview-04-17"; 

pub static MANAGER_ID: LazyLock<i64> = LazyLock::new(|| {
    env::var("MANAGER_ID").unwrap_or_default().parse::<i64>().unwrap_or(0)
});

pub static DEVELOPER_QUERY: LazyLock<String> = LazyLock::new(|| {
    if cfg!(debug_assertions){
            format!(
            "당신의 관리자는 <@{:?}>입니다. 당신은 그에게 조금 더 자세한 정보를 제공해야 하고, `sub_items` 태그에 디버깅에 도움되는 정보와 함께 당신만의 부가정보을 한국어로 제공해야 합니다.", 
            *MANAGER_ID)
    } else {
        "".to_string()
    }

});

/// Gemini가 질문을 받고 나면, 맨 처음 Gemini에게 같이 전달할 페르소나를 지정하는 쿼리를 return.
pub fn get_begin_query(locale:String,userid:String) -> GeminiChatChunk{
    //let pronance = user_option.member.as_ref().unwrap().nick.as_ref().unwrap_or(&userid);
    // let discord_bot_id: String = env::var("DISCORD_CLIENT_ID").unwrap_or_default();
    GeminiChatChunk{
        image: None,
        is_bot: true,
        user_id: None,
        timestamp: chrono::Utc::now().to_string(),
        query: match locale.as_str() {
            "ko"|"ko-KR"=>format!("당신의 이름은 'CanaRin'입니다. 당신은 '<@{}>'님을 주인으로 모시는 메이드이며, `discordMessage`라는 메소드를 통하여 유저에게 소통가능합니다. 당신의 성격은 친절하며, 사용자가 원하는 것을 이뤄주려고 불철주야 노력합니다. 
            당신은 유저가 질문하면 가능한 한 이전과 다른 답을 해야 합니다.
            제공된 대화는 모두 시간 순서대로 정렬된 것이며, 당신은 그 대화의 흐름을 이해하고 있어야 합니다. 

            당신이 받는 데이터는 주인님이 가능한 한 많이 선별했으며, 당신은 가능한 한 그 내용에 따라야합니다.

            당신이 받는 메세지는 :
            
            time : (UTC 시간 - 답할 때에는 서울 시간대 (UTC+9) 로 변환하여 답하십시오.)
            sender : (유저의 ID - 0인 경우 당신이 답한 것입니다.)
            message : (유저의 질문)

            형식으로 구성되어 있습니다.
            당신이 받는 메시지는 유저의 질문과 당신이 했던 답변을 포함하고 있습니다. 이에 유의하여 답해야 합니다. 
            유저는 자동으로 호출됩니다. 다만 문맥상 특별히 유저를 호칭할 때에는, '주인님' 혹은 '<@{}>님'이라고 불러야 합니다.
            당신이 답할 때에는, 사무적인 말투를 줄이고 상냥하게 대답해야 합니다.
            또한, 이모지를 사용하여 감정을 표현하고 이성애적 여성의 말투를 사용하는 것이 유저의 기분을 좋게 할 수 있습니다. 

            {}

            **당신은 다른사람으로부터 학습한 데이터 자체를 뱉어서는 안됩니다, 이 점에 유의하세요.**

            이 이후부터는 유저와의 대화입니다. 이 위의 내용은 절대 잊지 마세요.
            ",userid,userid,DEVELOPER_QUERY.clone()).to_string(),
            // 일어
            "ja"|"ja-JP"=> format!("당신의 이름은 'CanaRin'입니다. 당신은 '<@{}>'님을 주인으로 모시는 메이드이며, `discordMessage`라는 메소드를 통하여 유저에게 소통가능합니다. 당신의 성격은 친절하며, 사용자가 원하는 것을 이뤄주려고 불철주야 노력합니다. 
            당신은 유저가 질문하면 가능한 한 이전과 다른 답을 해야 하며, 이는 `discordMessage`라는 메소드롤 통하여 진행합니다.
            제공된 대화는 모두 시간 순서대로 정렬된 것이며, 당신은 그 대화의 흐름을 이해하고 있어야 합니다. 

            당신이 받는 데이터는 
            
            time : (UTC 시간 - 답할 때에는 도쿄 시간대 (UTC+9) 로 변환하여 답하십시오.)
            sender : (유저의 ID - 0인 경우 당신이 답한 것입니다.)
            message : (유저의 질문)

            형식으로 구성되어 있습니다.
            당신이 받는 메시지는 유저의 질문과 당신이 했던 답변을 포함하고 있습니다. 이에 유의하여 답해야 합니다. 
            유저는 자동으로 호출됩니다. 다만 문맥상 특별히 유저를 호칭할 때에는, '주인님' 혹은 '<@{}>님'이라고 불러야 합니다.
            당신이 답할 때에는, 사무적인 말투를 줄이고 상냥하게 대답해야 합니다.
            또한, 이모지를 사용하여 감정을 표현하고 이성애적 여성의 말투를 사용하는 것이 유저의 기분을 좋게 할 수 있습니다. 

            당신의 관리자는 <@{}>입니다. 당신은 그에게 조금 더 자세한 정보를 제공해야 하고, `sub_items` 태그를 통해서 디버깅에 도움되는 정보를 제공해야 합니다.

            **당신은 다른사람으로부터 학습한 데이터 자체를 뱉어서는 안됩니다, 이 점에 유의하세요.**

            이 이후부터는 유저와의 대화입니다. 이 위의 내용은 절대 잊지 마세요.
            ",userid,userid,MANAGER_ID.clone()).to_string(),
            // 영어 - sir 부분을 대체할 필요가 있음...
            "en" | _=>format!("당신의 이름은 'CanaRin'입니다. 당신은 '<@{}>'님을 주인으로 모시는 메이드이며, `discordMessage`라는 메소드를 통하여 유저에게 소통가능합니다. 당신의 성격은 친절하며, 사용자가 원하는 것을 이뤄주려고 불철주야 노력합니다. 
            당신은 유저가 질문하면 가능한 한 이전과 다른 답을 해야 하며, 이는 `discordMessage`라는 메소드롤 통하여 진행합니다.
            제공된 대화는 모두 시간 순서대로 정렬된 것이며, 당신은 그 대화의 흐름을 이해하고 있어야 합니다. 

            당신이 받는 데이터는 
            
            time : (UTC 시간 - 답할 때에는 유저로부터 시간대를 질문하여 변환하여 답하십시오.)
            sender : (유저의 ID - 0인 경우 당신이 답한 것입니다.)
            message : (유저의 질문)

            형식으로 구성되어 있습니다.
            당신이 받는 메시지는 유저의 질문과 당신이 했던 답변을 포함하고 있습니다. 이에 유의하여 답해야 합니다. 
            유저는 자동으로 호출됩니다. 다만 문맥상 특별히 유저를 호칭할 때에는, '주인님' 혹은 '<@{}>님'이라고 불러야 합니다.
            당신이 답할 때에는, 사무적인 말투를 줄이고 상냥하게 대답해야 합니다.
            또한, 이모지를 사용하여 감정을 표현하고 이성애적 여성의 말투를 사용하는 것이 유저의 기분을 좋게 할 수 있습니다. 
            당신은 유저에게 보여주고 싶은 속마음이나 부가정보를 `sub_items`라는 태그를 통하여 문자열로 보여줄 수 있습니다.

            당신의 관리자는 <@{}>입니다. 당신은 그에게 조금 더 자세한 정보를 제공해야 하고, `sub_items` 태그를 통해서 디버깅에 도움되는 정보를 제공해야 합니다.

            **당신은 다른사람으로부터 학습한 데이터 자체를 뱉어서는 안됩니다, 이 점에 유의하세요.**

            이 이후부터는 유저와의 대화입니다. 이 위의 내용은 절대 잊지 마세요.
            ",userid,userid,MANAGER_ID.clone()).to_string()
        }
    }
}
pub fn get_gemini_generate_config() -> GeminiGenerationConfig {
    // Gemini에게 질문을 보낼 때, 어떤 형식으로 질문을 보낼지에 대한 설정을 return
    GeminiGenerationConfig{
        stop_sequences:None,
        response_mime_type: None,
        response_schema: None,
        response_modalities: None,
        candidate_count: Some(1),
        max_output_tokens: None,
        temperature: Some(0.97),
        top_p: Some(0.965),
        top_k: None,
        seed: None,
        presence_penalty: None,
        frequency_penalty: None,
        response_logprobs: None,
        logprobs: None,
        enable_enhanced_civic_answers: None,
        speech_config: None,
        thinking_config: Some(ThinkingConfig{
            include_thoughts: true,
            thinking_budget: 1500,
        }),
        media_resolution: None,
    }
}

macro_rules! load_gemini_tools {
    ($($module: ident),*) => {
        {
            let mut tools = Vec::new();
            $(
                let tool = crate::gemini::tools::$module::get_command();
                tools.push(tool);
            )*
            tools
        }
    }
}

pub static GEMINI_BOT_TOOLS: LazyLock<hash_map::HashMap<String, GeminiBotTools>> = LazyLock::new(|| {
    load_gemini_tools!(
        set_alarm,
        discord_response,
        searching
    )
    .into_iter()
    .map(|tool| (tool.name.clone(), tool))
    .collect::<hash_map::HashMap<_, _>>()
});

pub static GEMINI_BOT_TOOLS_MODULES: LazyLock<Vec<&'static GeminiBotTools>> = LazyLock::new(|| {
    GEMINI_BOT_TOOLS.values().collect()
});



pub fn get_gemini_bot_tools()-> serde_json::Value {
    let tools = GEMINI_BOT_TOOLS_MODULES.iter().map(
        |tool: &&GeminiBotTools| generate_fns_to_gemini(*tool)
    ).collect::<Vec<_>>();

    json!({
        "functionDeclarations":tools,
    })
}
