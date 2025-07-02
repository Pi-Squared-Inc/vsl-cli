#![allow(unused)]

pub fn split_with_quotes(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes1 = false; // In quotes of first order, i.e. `'--opt <val>'``
    let mut in_quotes2 = false; // In quotes of second order, i.e. `--opt='<val1> <val2>'`
    let mut chars = input.chars().peekable();
    let mut prev_ch = '\0';

    while let Some(ch) = chars.next() {
        match ch {
            '\'' => {
                if in_quotes1 {
                    // Ending a quote - add the current token (don't trim quoted content)
                    result.push(current.clone());
                    current.clear();
                    in_quotes1 = false;
                } else if in_quotes2 {
                    current.push(ch);
                    in_quotes2 = false;
                } else {
                    if prev_ch == '=' {
                        // Special case: the argument value is enclosed into quotations: `--opt='<val1> <val2>'`
                        current.push(ch);
                        in_quotes2 = true;
                    } else {
                        // Starting a quote: general case - first add any pending unquoted content
                        if !current.is_empty() {
                            result.push(current.trim().to_string());
                            current.clear();
                        }
                        in_quotes1 = true;
                    }
                }
            }
            ' ' if !(in_quotes1 || in_quotes2) => {
                // Space outside quotes - end current token if it's not empty
                if !current.is_empty() {
                    result.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => {
                // Regular character or space inside quotes
                if in_quotes1 || in_quotes2 {
                    // Inside quotes: preserve all characters including whitespace
                    current.push(ch);
                } else if !ch.is_whitespace() {
                    // Outside quotes: only add non-whitespace characters
                    current.push(ch);
                }
            }
        }
        prev_ch = ch;
    }

    // Add the last token if it exists
    if !current.is_empty() {
        result.push(current.trim().to_string());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_splitting() {
        let result = split_with_quotes("hello world");
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_quoted_strings() {
        let result = split_with_quotes("hello 'world test' end");
        assert_eq!(result, vec!["hello", "world test", "end"]);
    }

    #[test]
    fn test_multiple_quotes() {
        let result = split_with_quotes("'first quote' 'second quote'");
        assert_eq!(result, vec!["first quote", "second quote"]);
    }

    #[test]
    fn test_whitespace_handling() {
        let result = split_with_quotes("'  spaced  content  '");
        assert_eq!(result, vec!["  spaced  content  "]);
    }

    #[test]
    fn test_empty_quotes() {
        let result = split_with_quotes("before '' after");
        assert_eq!(result, vec!["before", "", "after"]);
    }

    #[test]
    fn test_arg_eq_quotes() {
        let result = split_with_quotes("before --key='val1 val2' after");
        assert_eq!(result, vec!["before", "--key='val1 val2'", "after"]);
    }

    #[test]
    fn test_empty_arg_eq_quotes() {
        let result = split_with_quotes("before --key='' after");
        assert_eq!(result, vec!["before", "--key=''", "after"]);
    }

    #[test]
    fn test_spaced_arg_eq_quotes() {
        let result = split_with_quotes("before --key='   val1   val2   ' after");
        assert_eq!(result, vec!["before", "--key='   val1   val2   '", "after"]);
    }
}
