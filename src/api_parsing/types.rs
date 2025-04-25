use chrono::{DateTime, Utc};
use serde::Deserialize;

#[allow(unused)]
#[derive(Clone, Deserialize, Debug, Default)]
pub struct ProblemModelItem {
    pub slope: Option<f64>,
    pub intercept: Option<f64>,
    pub variance: Option<f64>,
    pub difficulty: Option<i64>,
    pub discrimination: Option<f64>,
    pub irt_loglikelihood: Option<f64>,
    pub irt_users: Option<i64>,
    pub is_experimental: Option<bool>,
}

#[allow(unused)]
#[derive(Clone, Deserialize, Debug, Default)]
pub struct ProblemItem {
    pub id: String,
    pub contest_id: String,
    pub problem_index: String,
    pub name: String,
    pub title: String,
}

#[allow(unused)]
#[derive(Clone, Deserialize, Debug)]
pub struct SubmissionItem {
    pub id: i64,
    pub epoch_second: i64,
    pub problem_id: String,
    pub contest_id: String,
    pub user_id: String,
    pub language: String,
    pub point: f64,
    pub length: i64,
    pub result: String,
    pub execution_time: Option<i64>,
}

#[allow(unused)]
#[derive(Clone, Deserialize, Debug)]
pub struct ContestItem {
    pub start_time: DateTime<Utc>,
    pub name: String,
    pub duration: u32,
    pub rated_range: String,
    pub url: String,
}
