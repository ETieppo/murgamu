#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MurTlsVersion {
	Tls12,
	#[default]
	Tls13,
}
