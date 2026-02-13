use super::*;

#[test]
fn test_mock_service_basic() {
	let mock = MockService::new();
	mock.set_return_value("get_user", serde_json::json!({"id": 1, "name": "John"}));

	let result: Option<serde_json::Value> = mock.call("get_user");
	assert!(result.is_some());
	assert_eq!(result.unwrap()["name"], "John");
	assert!(mock.was_called("get_user"));
	assert_eq!(mock.call_count("get_user"), 1);
}

#[test]
fn test_mock_service_call_tracking() {
	let mock = MockService::new();
	mock.set_return_value("method", serde_json::json!(null));

	let _: Option<()> = mock.call("method");
	let _: Option<()> = mock.call("method");
	let _: Option<()> = mock.call("method");

	assert_eq!(mock.call_count("method"), 3);
	assert!(mock.was_called_times("method", 3));

	mock.reset_calls();
	assert_eq!(mock.call_count("method"), 0);
}

#[test]
fn test_mock_service_builder() {
	let mock = MockServiceBuilder::<String>::new()
		.with_name("TestMock")
		.returns_for("greet", "Hello, World!")
		.build();

	let result: Option<String> = mock.call("greet");
	assert_eq!(result, Some("Hello, World!".to_string()));
}

#[test]
fn test_mock_service_expectations() {
	let mock = MockServiceBuilder::<String>::new()
		.expect_call("method_a")
		.times(2)
		.returns("value_a")
		.expect_call("method_b")
		.at_least(1)
		.returns("value_b")
		.build();

	let _: Option<String> = mock.call("method_a");
	let _: Option<String> = mock.call("method_a");
	let _: Option<String> = mock.call("method_b");

	assert!(mock.verify().is_ok());
}

#[test]
fn test_mock_service_expectations_fail() {
	let mock = MockServiceBuilder::<String>::new()
		.expect_call("method")
		.times(2)
		.returns("value")
		.build();

	let _: Option<String> = mock.call("method");

	assert!(mock.verify().is_err());
}

#[test]
fn test_mock_guard_allow() {
	let guard = MockGuard::new().allow_all();
	assert_eq!(guard.call_count(), 0);
}

#[test]
fn test_mock_guard_builder() {
	let guard = MockGuardBuilder::new()
		.name("TestGuard")
		.require_header("Authorization")
		.build();

	assert_eq!(guard.name, "TestGuard");
}

#[test]
fn test_mock_middleware_pass_through() {
	let middleware = MockMiddleware::new();
	assert!(middleware.pass_through);
	assert_eq!(middleware.call_count(), 0);
}

#[test]
fn test_mock_middleware_blocking() {
	let middleware = MockMiddleware::new().blocking();
	assert!(!middleware.pass_through);
}

#[test]
fn test_mock_interceptor() {
	let interceptor = MockInterceptor::new();
	assert_eq!(interceptor.call_count(), 0);
	assert_eq!(interceptor.name, "MockInterceptor");
}

#[test]
fn test_expected_call_count() {
	let exactly = ExpectedCallCount::Exactly(3);
	let at_least = ExpectedCallCount::AtLeast(1);
	let at_most = ExpectedCallCount::AtMost(5);
	let between = ExpectedCallCount::Between(2, 4);
	let never = ExpectedCallCount::Never;

	match exactly {
		ExpectedCallCount::Exactly(n) => assert_eq!(n, 3),
		_ => panic!("Expected Exactly variant"),
	}

	match at_least {
		ExpectedCallCount::AtLeast(n) => assert_eq!(n, 1),
		_ => panic!("Expected AtLeast variant"),
	}

	match at_most {
		ExpectedCallCount::AtMost(n) => assert_eq!(n, 5),
		_ => panic!("Expected AtMost variant"),
	}

	match between {
		ExpectedCallCount::Between(min, max) => {
			assert_eq!(min, 2);
			assert_eq!(max, 4);
		}
		_ => panic!("Expected Between variant"),
	}

	match never {
		ExpectedCallCount::Never => {}
		_ => panic!("Expected Never variant"),
	}
}

#[test]
fn test_mock_call() {
	let call = MockCall::new("test_method");
	assert_eq!(call.method, "test_method");
	assert!(call.args.is_none());

	let call_with_args = MockCall::with_args("test_method", serde_json::json!({"key": "value"}));
	assert_eq!(call_with_args.method, "test_method");
	assert!(call_with_args.args.is_some());
}
