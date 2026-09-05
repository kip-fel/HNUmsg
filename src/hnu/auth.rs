use anyhow::{anyhow, Result};

use hnu_query::cas::{
    login::{AccountIssue, CasToken},
    tfa::{SMSResult, TFAToken, VerifyResult},
};

use crate::config::Config;

pub enum LoginResult {
    Ready(CasToken),

    WaitingForSms {
        token: TFAToken,
        phone: String,
    },
}

pub async fn login(config: &Config) -> Result<LoginResult> {
    let result =
        CasToken::acquire_by_login(&config.stu_id, &config.password)
            .await;

    match result {
        Ok(cas_token) => {
            Ok(LoginResult::Ready(cas_token))
        }

        Err(hnu_query::Error::Other(AccountIssue::TFARequired(token))) => {
            let phone = token.phone().to_string();

            let sms_result = token
                .send_sms()
                .await
                .map_err(|e| anyhow!("{}", e))?;

            match sms_result {
                SMSResult::Success => {
                    Ok(LoginResult::WaitingForSms {
                        token,
                        phone,
                    })
                }

                SMSResult::Valid => {
                    Ok(LoginResult::WaitingForSms {
                        token,
                        phone,
                    })
                }

                SMSResult::Other(message) => {
                    Err(anyhow!(
                        "短信发送失败：{}",
                        message
                    ))
                }
            }
        }

        Err(hnu_query::Error::Other(AccountIssue::PasswordError)) => {
            Err(anyhow!("湖南大学统一身份认证：用户名或密码错误"))
        }

        Err(hnu_query::Error::Other(
            AccountIssue::PasswordShouldChange,
        )) => {
            Err(anyhow!(
                "湖南大学账号要求修改密码，请先前往个人门户修改密码"
            ))
        }

        Err(hnu_query::Error::Other(AccountIssue::AccountLocked)) => {
            Err(anyhow!(
                "湖南大学账号因多次输错密码被锁定"
            ))
        }

        Err(error) => {
            Err(anyhow!(
                "湖南大学统一身份认证失败：{}",
                error
            ))
        }
    }
}

pub async fn verify_sms(
    token: TFAToken,
    code: &str,
) -> Result<SmsVerificationResult> {
    let code = code.trim();

    if code.is_empty() {
        return Err(anyhow!("短信验证码不能为空"));
    }

    let result = token
        .verify(code)
        .await
        .map_err(|e| anyhow!("{}", e))?;

    match result {
        VerifyResult::Success(cas_token) => {
            Ok(SmsVerificationResult::Success(cas_token))
        }

        VerifyResult::CodeError(new_token) => {
            let phone = new_token.phone().to_string();

            Ok(SmsVerificationResult::CodeError {
                token: new_token,
                phone,
            })
        }

        VerifyResult::Expired => {
            Ok(SmsVerificationResult::Expired)
        }
    }
}

pub enum SmsVerificationResult {
    Success(CasToken),

    CodeError {
        token: TFAToken,
        phone: String,
    },

    Expired,
}