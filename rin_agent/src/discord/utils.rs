use serenity::all::{Context, CreateInteractionResponse, Guild, GuildId, PartialGuild};

pub enum GuildInfo {
    Full(Guild),
    Partial(PartialGuild),
}

pub async fn get_guild_info(ctx: &Context, guild_id: GuildId) -> Result<GuildInfo, serenity::Error> {
    // Try to get from cache first
    if let Some(guild) = guild_id.to_guild_cached(&ctx.cache) {
      Ok(GuildInfo::Full(guild.clone()))
    } else {
        // If not in cache, fetch from API
        match ctx.http.get_guild(guild_id).await {
            Ok(guild) => {
              Ok(GuildInfo::Partial(guild.clone()))
            },
            Err(why) => {
                eprintln!("Error getting guild: {:?}", why);
                Err(serenity::Error::Other("Failed to fetch guild information"))
            },
        }
    }
}

#[derive(Debug,Clone)]
pub struct GuildCommandResponse {
  pub content: CreateInteractionResponse,
  pub do_not_send : bool, // If true, do not send the response immediately
}