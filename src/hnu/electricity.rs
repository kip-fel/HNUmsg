use anyhow::Result;
use hnu_query::xgxt::personal_info::Dormitory;

use super::dorm::validate_dormitory;

pub async fn get_electricity(dormitory: Dormitory) -> Result<String> {
    validate_dormitory(&dormitory)?;

    let result = hnu_query::wxpay::get_electricity(dormitory)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(result)
}