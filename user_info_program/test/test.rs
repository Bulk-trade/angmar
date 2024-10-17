mod expected_drift_program_id {
    solana_program::declare_id!("dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_check_program_id() {
        println!("Expected Program ID: {}", expected_drift_program_id::ID);
        println!("Actual Program ID: {}", drift_interface::ID);
        assert_eq!(expected_drift_program_id::ID, drift_interface::ID);
    }
}