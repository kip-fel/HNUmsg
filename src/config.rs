use anyhow::{Context, Result};
use std::env;

#[derive(Clone)]
pub struct Config {
    pub stu_id: String,
    pub password: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let stu_id =
            env::var("HNU_STU_ID").context("缺少环境变量 HNU_STU_ID")?;

        let password =
            env::var("HNU_PASSWORD").context("缺少环境变量 HNU_PASSWORD")?;

        if stu_id.trim().is_empty() {
            anyhow::bail!("HNU_STU_ID 不能为空");
        }

        if password.is_empty() {
            anyhow::bail!("HNU_PASSWORD 不能为空");
        }

        Ok(Self {
            stu_id,
            password,
        })
    }
}