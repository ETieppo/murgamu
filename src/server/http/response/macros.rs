#[macro_export]
macro_rules! mur_json {
    ($($json:tt)+) => {
        $crate::server::http::MurHttpResponse::json(serde_json::json!($($json)+))
    };
}

#[macro_export]
macro_rules! mur_text {
	($text:expr) => {
		$crate::server::http::MurHttpResponse::text($text)
	};
}

#[macro_export]
macro_rules! mur_html {
	($html:expr) => {
		$crate::server::http::MurHttpResponse::html($html)
	};
}
