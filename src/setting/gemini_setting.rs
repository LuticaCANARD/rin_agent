use serenity::model::prelude::*;

/// Gemini가 질문을 받고 나면, 맨 처음 Gemini에게 같이 전달할 페르소나를 지정하는 쿼리를 return.
async fn get_begin_query(locale:String) -> String {
    let mut loaded = "Gemini는 당신의 질문에 답변하기 위해 최선을 다할 것입니다. 질문을 입력하세요.";



    loaded.to_string()
}