#[macro_export]
macro_rules! assert_json_eq {
    ($left:expr, $right:expr) => {
        let left_value: serde_json::Value = serde_json::to_value(&$left).expect("Failed to serialize left value");
        let right_value: serde_json::Value = serde_json::to_value(&$right).expect("Failed to serialize right value");

        if left_value != right_value {
            panic!(
                "JSON values are not equal:\n  Left:\n{}\n  Right:\n{}",
                serde_json::to_string_pretty(&left_value).unwrap(),
                serde_json::to_string_pretty(&right_value).unwrap()
            );
        }
    };
    ($left:expr, $right:expr, $($arg:tt)+) => {
        let left_value: serde_json::Value = serde_json::to_value(&$left).expect("Failed to serialize left value");
        let right_value: serde_json::Value = serde_json::to_value(&$right).expect("Failed to serialize right value");

        if left_value != right_value {
            panic!(
                "{}\n  Left:\n{}\n  Right:\n{}",
                format_args!($($arg)+),
                serde_json::to_string_pretty(&left_value).unwrap(),
                serde_json::to_string_pretty(&right_value).unwrap()
            );
        }
    };
}

#[macro_export]
macro_rules! assert_status {
	($response:expr, $status:expr) => {
		assert_eq!(
			$response.status(),
			$status,
			"Expected status {}, got {}",
			$status,
			$response.status()
		);
	};
}

#[macro_export]
macro_rules! assert_success {
	($response:expr) => {
		assert!(
			$response.status().is_success(),
			"Expected success status, got {}",
			$response.status()
		);
	};
}
