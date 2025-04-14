use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrateMetrics {
    pub last_commit: Option<DateTime<Utc>>,
    pub issue_response_time: Option<Duration>,
    pub releases_per_year: f32,
    pub readme_length: usize,
    pub has_docs_rs: bool,
    pub has_examples: bool,
    pub cargo_audit_passed: bool,
    pub rustsec_vulnerabilities: usize,
    pub dependent_count: usize,
    pub recent_downloads: usize,
    pub total_downloads: usize,
    pub license: String,
}

pub struct CrateScorer {
    // 配置参数
    maintenance_threshold: Duration,
    issue_response_threshold: Duration,
    min_releases_per_year: f32,
    min_readme_length: usize,
    min_dependent_count: usize,
    recent_download_period: Duration,
    recommended_licenses: Vec<String>,
}

impl Default for CrateScorer {
    fn default() -> Self {
        Self {
            maintenance_threshold: Duration::days(90),
            issue_response_threshold: Duration::days(7),
            min_releases_per_year: 2.0,
            min_readme_length: 500,
            min_dependent_count: 10,
            recent_download_period: Duration::days(30),
            recommended_licenses: vec![
                "MIT".to_string(),
                "Apache-2.0".to_string(),
                "BSD-3-Clause".to_string(),
            ],
        }
    }
}

impl CrateScorer {
    pub fn new() -> Self {
        Self::default()
    }

    /// 评估维护活跃度
    /// 考虑因素：
    /// 1. 最近提交时间（权重：0.4）
    /// 2. issue 响应速度（权重：0.3）
    /// 3. 每年发布次数（权重：0.3）
    pub fn score_maintenance(&self, metrics: &CrateMetrics) -> f32 {
        let mut score = 0.0;
        let now = Utc::now();

        // 评估最近提交时间
        if let Some(last_commit) = metrics.last_commit {
            let days_since_commit = (now - last_commit).num_days() as f32;
            let commit_score = if days_since_commit <= self.maintenance_threshold.num_days() as f32 {
                1.0
            } else {
                (self.maintenance_threshold.num_days() as f32 / days_since_commit).min(1.0)
            };
            score += commit_score * 0.4;
        }

        // 评估 issue 响应速度
        if let Some(response_time) = metrics.issue_response_time {
            let response_score = if response_time <= self.issue_response_threshold {
                1.0
            } else {
                (self.issue_response_threshold.num_days() as f32 / response_time.num_days() as f32).min(1.0)
            };
            score += response_score * 0.3;
        }

        // 评估发布频率
        let release_score = (metrics.releases_per_year / self.min_releases_per_year).min(1.0);
        score += release_score * 0.3;

        score
    }

    /// 评估文档质量
    /// 考虑因素：
    /// 1. README 长度（权重：0.4）
    /// 2. docs.rs 文档（权重：0.3）
    /// 3. 示例代码（权重：0.3）
    pub fn score_documentation(&self, metrics: &CrateMetrics) -> f32 {
        let mut score = 0.0;

        // 评估 README 长度
        let readme_score = (metrics.readme_length as f32 / self.min_readme_length as f32).min(1.0);
        score += readme_score * 0.4;

        // 评估 docs.rs 文档
        if metrics.has_docs_rs {
            score += 0.3;
        }

        // 评估示例代码
        if metrics.has_examples {
            score += 0.3;
        }

        score
    }

    /// 评估安全性
    /// 考虑因素：
    /// 1. cargo-audit 检查（权重：0.6）
    /// 2. RustSec 漏洞数量（权重：0.4）
    pub fn score_security(&self, metrics: &CrateMetrics) -> f32 {
        let mut score = 0.0;

        // 评估 cargo-audit 结果
        if metrics.cargo_audit_passed {
            score += 0.6;
        }

        // 评估 RustSec 漏洞
        let vulnerability_score = if metrics.rustsec_vulnerabilities == 0 {
            1.0
        } else {
            (1.0 / (metrics.rustsec_vulnerabilities as f32 + 1.0)).min(1.0)
        };
        score += vulnerability_score * 0.4;

        score
    }

    /// 评估被依赖数
    /// 使用对数函数来平滑评分
    pub fn score_dependents(&self, metrics: &CrateMetrics) -> f32 {
        if metrics.dependent_count < self.min_dependent_count {
            return 0.0;
        }
        (metrics.dependent_count as f32).ln() / (self.min_dependent_count as f32).ln()
    }

    /// 评估下载趋势
    /// 计算近期下载量与总下载量的比值
    pub fn score_download_trend(&self, metrics: &CrateMetrics) -> f32 {
        if metrics.total_downloads == 0 {
            return 0.0;
        }
        (metrics.recent_downloads as f32 / metrics.total_downloads as f32).min(1.0)
    }

    /// 评估许可证
    /// 检查是否使用推荐的开源许可证
    pub fn score_license(&self, metrics: &CrateMetrics) -> f32 {
        if self.recommended_licenses.contains(&metrics.license) {
            1.0
        } else {
            0.5
        }
    }

    /// 计算总体评分
    /// 权重分配：
    /// - 维护活跃度：0.25
    /// - 文档质量：0.20
    /// - 安全性：0.15
    /// - 被依赖数：0.15
    /// - 下载趋势：0.15
    /// - 许可证：0.10
    pub fn calculate_total_score(&self, metrics: &CrateMetrics) -> f32 {
        let maintenance_score = self.score_maintenance(metrics);
        let documentation_score = self.score_documentation(metrics);
        let security_score = self.score_security(metrics);
        let dependents_score = self.score_dependents(metrics);
        let download_trend_score = self.score_download_trend(metrics);
        let license_score = self.score_license(metrics);

        maintenance_score * 0.25 +
        documentation_score * 0.20 +
        security_score * 0.15 +
        dependents_score * 0.15 +
        download_trend_score * 0.15 +
        license_score * 0.10
    }

    /// 生成详细的评分报告
    pub fn generate_score_report(&self, metrics: &CrateMetrics) -> HashMap<String, f32> {
        let mut report = HashMap::new();
        report.insert("maintenance".to_string(), self.score_maintenance(metrics));
        report.insert("documentation".to_string(), self.score_documentation(metrics));
        report.insert("security".to_string(), self.score_security(metrics));
        report.insert("dependents".to_string(), self.score_dependents(metrics));
        report.insert("download_trend".to_string(), self.score_download_trend(metrics));
        report.insert("license".to_string(), self.score_license(metrics));
        report.insert("total".to_string(), self.calculate_total_score(metrics));
        report
    }
} 