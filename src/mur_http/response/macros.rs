#[macro_export]
macro_rules! mur_json {
    ($($json:tt)+) => {
        $crate::mur_http::response::MurHttpResponse::json(serde_json::json!($($json)+))
    };
}

#[macro_export]
macro_rules! mur_text {
	($text:expr) => {
		$crate::mur_http::response::MurHttpResponse::text($text)
	};
}

#[macro_export]
macro_rules! mur_html {
	($html:expr) => {
		$crate::mur_http::response::MurHttpResponse::html($html)
	};
}
