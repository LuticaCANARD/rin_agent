use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
        .create_table(
            Table::create()
                .table(TbAlarmModel::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(TbAlarmModel::Id)
                    .big_integer()
                    .not_null()
                    .auto_increment()
                    .primary_key()
                )
                .col(
                    date_time(TbAlarmModel::Time)
                    .timestamp_with_time_zone()
                )
                .col(ColumnDef::new(TbAlarmModel::Message).text().not_null())
                .col(string_null(TbAlarmModel::RepeatCircle).string_len(100)) 
                .col(date_time_null(TbAlarmModel::RepeatEndAt))
                .col(ColumnDef::new(TbAlarmModel::CreatedAt)
                    .date_time()
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Expr::current_timestamp())
                    .take()
                )
                .col(date_time(TbAlarmModel::UpdatedAt)
                    .date_time()
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Expr::current_timestamp())
                    .take()
                )
                .to_owned(),
        ).await?;

        manager.create_table(
            Table::create()
                .table(TbImageAttachFile::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(TbImageAttachFile::ImageId)
                        .big_integer()
                        .not_null()
                        .auto_increment()
                        .primary_key()
                )
                .col(ColumnDef::new(TbImageAttachFile::FileSrc).text().not_null())
                .col(string_null(TbImageAttachFile::MimeType))
                .to_owned()
        ).await?;

        manager.create_table(
            Table::create()
                .table(TbAiContext::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(TbAiContext::Id)
                    .big_integer()
                    .not_null()
                    .auto_increment()
                    .primary_key()
                )
                .col(big_integer(TbAiContext::UserId))
                .col(ColumnDef::new(TbAiContext::Context).text().not_null())
                .col(timestamp_with_time_zone(TbAiContext::CreatedAt))
                .col(date_time(TbAiContext::UpdatedAt))
                .col(big_integer(TbAiContext::GuildId))
                .col(big_integer(TbAiContext::ChannelId))
                .col(boolean(TbAiContext::ByBot))
                .col(big_integer_null(TbAiContext::ImageFileId))
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-tb_ai_context-image_file_id")
                        .from(TbAiContext::Table, TbAiContext::ImageFileId)
                        .to(TbImageAttachFile::Table, TbImageAttachFile::ImageId)
                )
                .to_owned()
        ).await?;

        manager.create_table(
            Table::create()
                .table(TbDiscordAiContext::Table)
                .if_not_exists()
                .col(
                        ColumnDef::new(TbDiscordAiContext::Id)
                        .big_integer()
                        .not_null()
                        .auto_increment()
                        .primary_key()
                    )
                .col(big_integer(TbDiscordAiContext::GuildId))
                .col(big_integer(TbDiscordAiContext::RootMsg))
                .col(boolean(TbDiscordAiContext::UsingProModel))
                .col(
                    ColumnDef::new(TbDiscordAiContext::ParentContext)
                        .array(ColumnType::BigInteger)
                        .not_null()
                    )
                .to_owned()
        ).await?;

        manager.create_table(
            Table::create()
                .table(TbContextToMsgId::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(TbContextToMsgId::AiMsg)
                        .big_integer()
                        .not_null()
                        .primary_key()
                )
                .col(big_integer(TbContextToMsgId::AiContext))
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-tb_context_to_msg_id-ai_msg")
                        .from(TbContextToMsgId::Table, TbContextToMsgId::AiMsg)
                        .to(TbAiContext::Table, TbAiContext::Id)
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-tb_context_to_msg_id-ai_context")
                        .from(TbContextToMsgId::Table, TbContextToMsgId::AiContext)
                        .to(TbDiscordAiContext::Table, TbDiscordAiContext::Id)
                )
                .to_owned()
        ).await?;

        manager.create_table(
            Table::create()
                .table(TbDiscordMessageToAtContext::Table)
                .if_not_exists()
                .col(ColumnDef::new(TbDiscordMessageToAtContext::DiscordMessage).big_integer().not_null().primary_key())
                .col(big_integer(TbDiscordMessageToAtContext::AiMsgId))
                .col(timestamp_with_time_zone(TbDiscordMessageToAtContext::UpdateAt))
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-tb_discord_message_to_at_context-ai_msg_id")
                        .from(TbDiscordMessageToAtContext::Table, TbDiscordMessageToAtContext::AiMsgId)
                        .to(TbContextToMsgId::Table, TbContextToMsgId::AiMsg)
                )
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        
        manager
            .drop_table(Table::drop().table(TbDiscordMessageToAtContext::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TbContextToMsgId::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TbDiscordAiContext::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TbAiContext::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TbImageAttachFile::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TbAlarmModel::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TbAlarmModel {
    Table,
    Id,
    Time,
    Message,
    RepeatCircle,
    RepeatEndAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TbImageAttachFile {
    Table,
    ImageId,
    FileSrc,
    MimeType,
}

#[derive(DeriveIden)]
enum TbAiContext {
    Table,
    Id,
    UserId,
    Context,
    CreatedAt,
    UpdatedAt,
    GuildId,
    ChannelId,
    ByBot,
    ImageFileId,
}

#[derive(DeriveIden)]
enum TbDiscordAiContext {
    Table,
    Id,
    GuildId,
    RootMsg,
    UsingProModel,
    ParentContext,
}

#[derive(DeriveIden)]
enum TbContextToMsgId {
    Table,
    AiMsg,
    AiContext,
}

#[derive(DeriveIden)]
enum TbDiscordMessageToAtContext {
    Table,
    DiscordMessage,
    AiMsgId,
    UpdateAt,
}
