pub fn try_match_error_data(data: &str) -> eyre::Result<()> {
    match data {
        "37530076" => eyre::bail!("Immutable period has not yet passed"),
        "c0f96105" => eyre::bail!("Operator is still active"),
        "7ea8ac56" => eyre::bail!("Operator is already paused"),
        "f2a5f75a" => eyre::bail!("Operator is already unpaused"),
        "8f54ee5c" => eyre::bail!("Not a network"),
        _ => Ok(()),
    }
}
