//! Rule-based detection patterns.

/// Detection rules for specific threat patterns
pub struct DetectionRules;

impl DetectionRules {
    /// Check if a file extension change is suspicious
    pub fn is_suspicious_extension_change(old_ext: &str, new_ext: &str) -> bool {
        // Known document extensions being changed to unknown ones
        let document_exts = [
            "doc", "docx", "xls", "xlsx", "ppt", "pptx", "pdf",
            "jpg", "jpeg", "png", "gif", "bmp", "tiff",
            "mp4", "mov", "avi", "mkv", "mp3", "wav",
            "zip", "rar", "7z", "tar", "gz",
            "psd", "ai", "indd", "ae", "prproj",
            "txt", "csv", "json", "xml", "html",
        ];

        let old_lower = old_ext.to_lowercase();
        let new_lower = new_ext.to_lowercase();

        // Document extension changing to non-document extension
        document_exts.contains(&old_lower.as_str()) && !document_exts.contains(&new_lower.as_str())
    }

    /// Check if a process name is commonly associated with ransomware
    pub fn is_suspicious_process(process_name: &str) -> bool {
        let name_lower = process_name.to_lowercase();
        let suspicious_patterns = [
            "crypt", "ransom", "locker", "wanna",
        ];

        suspicious_patterns.iter().any(|p| name_lower.contains(p))
    }

    /// Calculate entropy of data (higher entropy = more likely encrypted)
    pub fn calculate_entropy(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mut freq = [0u64; 256];
        for &byte in data {
            freq[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &count in &freq {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Check if entropy indicates encryption (> 7.9 bits/byte is very high)
    pub fn is_high_entropy(entropy: f64) -> bool {
        entropy > 7.9
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy() {
        // Low entropy (repeated data)
        let low = vec![0u8; 1000];
        assert!(DetectionRules::calculate_entropy(&low) < 1.0);

        // High entropy (random-like data)
        let high: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        assert!(DetectionRules::calculate_entropy(&high) > 7.0);
    }

    #[test]
    fn test_suspicious_extension() {
        assert!(DetectionRules::is_suspicious_extension_change("docx", "encrypted"));
        assert!(!DetectionRules::is_suspicious_extension_change("docx", "pdf"));
    }
}
