
pub fn split_text_by_length_and_markdown(text: &str, max_length: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_chunk = String::new();
    let mut is_code_block = false;
    let mut code_language = String::new();

    for line in text.lines() {
        if line.starts_with("```") {
            is_code_block = !is_code_block;
            if is_code_block {
                code_language = line[3..].trim().to_string();
            }
        }
        if current_chunk.len() + line.len() + 1 > max_length {
            if is_code_block {
                current_chunk.push_str("```");
            }
            result.push(current_chunk.clone());
            
            current_chunk.clear();
            if is_code_block {
                current_chunk.push_str(&format!("```{}", code_language));
            }
        }
        if !current_chunk.is_empty() {
            current_chunk.push('\n');
        }

        current_chunk.push_str(line);
    }

    if !current_chunk.is_empty() {
        result.push(current_chunk);
    }

    result
}