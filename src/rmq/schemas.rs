use lapin::ExchangeKind;

#[derive(Debug)]
pub struct Exchange<'a> {
    pub name: &'a str,
    pub exchange_type: ExchangeKind,
}

impl<'a> Exchange<'a> {
    pub fn new(
        name: &'a str,
        exchange_type: &str,
    ) -> Self {
        let lowercased_exchange_type = exchange_type.to_ascii_lowercase();
        let exchange_type = match lowercased_exchange_type.as_str() {
            "direct" => ExchangeKind::Direct,
            "fanout" => ExchangeKind::Fanout,
            "headers" => ExchangeKind::Headers,
            _ => ExchangeKind::Topic,
        };
        Self {
            name,
            exchange_type,
        }
    }
}

#[derive(Debug, Default)]
pub struct Queue<'a> {
    pub name: &'a str,
    pub routing_key: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_simple_exchange() {
        let name = "test_exchange";
        let exchange_type = ExchangeKind::Direct;

        let result = Exchange {
            name,
            exchange_type: exchange_type.clone(),
        };

        assert_eq!(result.name, name);
        assert_eq!(result.exchange_type, exchange_type.to_owned());
    }

    #[test]
    fn create_from_new_exchange() {
        let name = "test_exchange";
        let type_1 = "direct";
        let type_2 = "fanout";
        let type_3 = "topic";
        let type_4 = "headers";

        assert_eq!(
            Exchange::new(name, type_1).exchange_type,
            ExchangeKind::Direct
        );
        assert_eq!(
            Exchange::new(name, type_2).exchange_type,
            ExchangeKind::Fanout
        );
        assert_eq!(
            Exchange::new(name, type_3).exchange_type,
            ExchangeKind::Topic
        );
        assert_eq!(
            Exchange::new(name, type_4).exchange_type,
            ExchangeKind::Headers
        );
    }

    #[test]
    fn create_queue() {
        let name = "test_queue";
        let routing_key = "test_routing_key";

        let result = Queue { name, routing_key };
        assert_eq!(result.name, name);
        assert_eq!(result.routing_key, routing_key);
    }
}
