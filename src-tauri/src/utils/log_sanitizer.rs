//! 日志脱敏模块
//! 对敏感信息进行智能遮盖，保护用户隐私的同时保留调试价值

use regex::Regex;

/// 日志脱敏器
pub struct LogSanitizer {
    /// 邮箱正则表达式
    email_regex: Regex,
    /// 键值对形式的敏感信息（token/secret/api_key 等）
    sensitive_kv_regex: Regex,
    /// Authorization: Bearer <token> 形式
    bearer_regex: Regex,
    /// 用户主目录正则表达式
    user_home_regex: Regex,
    /// Windows 用户目录正则表达式
    windows_user_regex: Regex,
}

impl Default for LogSanitizer {
    fn default() -> Self {
        Self {
            email_regex: Regex::new(r"(?i)[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
                .unwrap(),
            // 兼容 JSON / Header / querystring 等多种写法：
            // - "access_token":"<value>"
            // - access_token=<value>
            // - client_secret: <value>
            // 说明：只要 value 长度足够长（>=20）就进行脱敏，减少误伤。
            sensitive_kv_regex: Regex::new(
                r#"(?ix)
                (?P<prefix>
                    (?:\\)?["']?
                    (?:
                        key|
                        token|
                        secret|
                        api[-_]?key|
                        access[-_]?token|
                        id[-_]?token|
                        refresh[-_]?token|
                        client[-_]?secret
                    )
                    (?:\\)?["']?
                )
                (?P<sep>\s*(?::|=)\s*(?:\\)?["']?)
                (?P<key>[A-Za-z0-9._~+/=-]{20,})
                "#,
            )
            .unwrap(),
            bearer_regex: Regex::new(r"(?i)(?P<prefix>Bearer\s+)(?P<token>[A-Za-z0-9._~+/=-]{20,})")
                .unwrap(),
            user_home_regex: Regex::new(r"(?P<prefix>/home/[^/]+)").unwrap(),
            windows_user_regex: Regex::new(r"C:\\\\Users\\\\[^\\\\]+").unwrap(),
        }
    }
}

impl LogSanitizer {
    /// 创建新的脱敏器实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 对字符串进行脱敏处理
    pub fn sanitize(&self, input: &str) -> String {
        let mut result = input.to_string();

        // 1. 脱敏邮箱地址
        result = self.sanitize_email(&result);

        // 2. 脱敏用户路径
        result = self.sanitize_paths(&result);

        // 3. 脱敏敏感键值对（token/secret/api_key 等）
        result = self.sanitize_sensitive_kv(&result);

        // 4. 脱敏 Bearer Token
        result = self.sanitize_bearer_token(&result);

        result
    }

    /// 智能邮箱脱敏 - 保留首尾字符，中间固定用 "***" 替代
    ///
    /// 策略：
    /// - 1个字符：保留原样
    /// - 2个字符：显示首字符 + *
    /// - 3个及以上：显示首字符 + "***" + 尾字符
    ///
    /// # 示例
    /// ```
    /// "a@domain.com" → "a@domain.com"
    /// "ab@domain.com" → "a*@domain.com"
    /// "user@domain.com" → "u***r@domain.com"
    /// "very.long.email@domain.com" → "v***l@domain.com"
    /// ```
    pub fn sanitize_email(&self, input: &str) -> String {
        self.email_regex
            .replace_all(input, |caps: &regex::Captures| {
                let email = &caps[0];

                let at_pos = email.find('@').unwrap_or(0);
                let (local_part, domain_with_at) = email.split_at(at_pos);

                match local_part.len() {
                    0 | 1 => email.to_string(),
                    2 => {
                        let first_char = local_part.chars().next().unwrap_or('_');
                        format!("{}*{}", first_char, domain_with_at)
                    }
                    _ => {
                        let first_char = local_part.chars().next().unwrap_or('_');
                        let last_char = local_part.chars().last().unwrap_or('_');
                        format!("{}***{}{}", first_char, last_char, domain_with_at)
                    }
                }
            })
            .to_string()
    }

    /// 路径脱敏 - 隐藏用户主目录部分
    ///
    /// # 示例
    /// ```
    /// "/home/user/.antigravity-agent" → "~/.antigravity-agent"
    /// "/home/user/Documents/file.txt" → "~/Documents/file.txt"
    /// "C:\\Users\\Kiki\\AppData" → "~\\AppData"
    /// "C:\\Users\\Kiki\\AppData\\Roaming\\Antigravity" → "~\\AppData\\Roaming\\Antigravity"
    /// ```
    pub fn sanitize_paths(&self, input: &str) -> String {
        let mut result = input.to_string();

        // 处理 Linux/Unix 路径
        result = self
            .user_home_regex
            .replace_all(&result, |_caps: &regex::Captures| "~")
            .to_string();

        // 处理 Windows 路径 - 修正正则表达式匹配用户名
        result = self
            .windows_user_regex
            .replace_all(&result, |_caps: &regex::Captures| "~")
            .to_string();

        // 额外处理一些可能遗漏的路径格式
        if result.contains("C:\\Users\\") {
            // 使用更简单的替换方式
            result = regex::Regex::new(r"C:\\\\Users\\\\[^\\\\]+")
                .unwrap()
                .replace_all(&result, "~")
                .to_string();
        }

        result
    }

    /// 脱敏常见键值对敏感字段（token/secret/api_key 等）
    pub fn sanitize_sensitive_kv(&self, input: &str) -> String {
        self.sensitive_kv_regex
            .replace_all(input, |caps: &regex::Captures| {
                let prefix = &caps["prefix"];
                let sep = &caps["sep"];
                let key = &caps["key"];

                let visible_len = std::cmp::min(4, key.len());
                let masked_len = key.len().saturating_sub(visible_len);

                if key.len() <= 4 {
                    format!("{}{}{}", prefix, sep, key)
                } else {
                    let visible_part = &key[..visible_len];
                    let masked_part = "*".repeat(masked_len);
                    format!("{}{}{}{}", prefix, sep, visible_part, masked_part)
                }
            })
            .to_string()
    }

    /// 脱敏 Bearer Token
    pub fn sanitize_bearer_token(&self, input: &str) -> String {
        self.bearer_regex
            .replace_all(input, |caps: &regex::Captures| {
                let prefix = &caps["prefix"];
                let token = &caps["token"];

                let visible_len = std::cmp::min(4, token.len());
                let masked_len = token.len().saturating_sub(visible_len);

                if token.len() <= 4 {
                    format!("{}{}", prefix, token)
                } else {
                    let visible_part = &token[..visible_len];
                    let masked_part = "*".repeat(masked_len);
                    format!("{}{}{}", prefix, visible_part, masked_part)
                }
            })
            .to_string()
    }
}

/// 对日志消息进行脱敏处理的便捷函数
pub fn sanitize_log_message(message: &str) -> String {
    let sanitizer = LogSanitizer::new();
    sanitizer.sanitize(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_email_masks_local_part() {
        let s = LogSanitizer::new();
        assert_eq!(s.sanitize_email("a@domain.com"), "a@domain.com");
        assert_eq!(s.sanitize_email("ab@domain.com"), "a*@domain.com");
        assert_eq!(s.sanitize_email("user@domain.com"), "u***r@domain.com");
        assert_eq!(
            s.sanitize_email("very.long.email@domain.com"),
            "v***l@domain.com"
        );
    }

    #[test]
    fn sanitize_sensitive_kv_handles_json_style_tokens() {
        let s = LogSanitizer::new();
        let input = r#"{\"access_token\":\"abcdefghijklmnopqrstuvwxyz012345\"}"#;
        let out = s.sanitize_sensitive_kv(input);
        assert!(out.contains(r#"\"access_token\":\"abcd"#));
        assert!(!out.contains("abcdefghijklmnopqrstuvwxyz012345"));
    }

    #[test]
    fn sanitize_sensitive_kv_handles_querystring_style_tokens() {
        let s = LogSanitizer::new();
        let input = "client_secret=GOCSPX-abcdefghijklmnopqrstuvwxyz012345";
        let out = s.sanitize_sensitive_kv(input);
        assert!(out.contains("client_secret=GOCS"));
        assert!(!out.contains("GOCSPX-abcdefghijklmnopqrstuvwxyz012345"));
    }

    #[test]
    fn sanitize_bearer_token_masks_authorization_header() {
        let s = LogSanitizer::new();
        let input = "Authorization: Bearer abcdefghijklmnopqrstuvwxyz012345";
        let out = s.sanitize_bearer_token(input);
        assert!(out.contains("Authorization: Bearer abcd"));
        assert!(!out.contains("abcdefghijklmnopqrstuvwxyz012345"));
    }
}
