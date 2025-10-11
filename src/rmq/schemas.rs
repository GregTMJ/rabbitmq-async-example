use lapin::ExchangeKind;

#[derive(Debug)]
pub struct Exchange<'a> {
    pub name: &'a str,
    pub routing_key: &'a str,
    pub exchange_type: ExchangeKind,
}

impl<'a> Exchange<'a> {
    pub fn new(name: &'a str, routing_key: &'a str, exchange_type: &str) -> Self {
        let exchange_type = match exchange_type {
            "direct" => ExchangeKind::Direct,
            "fanout" => ExchangeKind::Fanout,
            _ => ExchangeKind::Topic,
        };
        Self {
            name,
            routing_key,
            exchange_type,
        }
    }
}

#[derive(Debug, Default)]
pub struct Queue<'a> {
    pub name: &'a str,
}
