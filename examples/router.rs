use prefix_tree_map::{CaptureMap, KeyPart, PrefixTreeMap, PrefixTreeMapBuilder};

struct Router {
    table: PrefixTreeMap<&'static str, Param, Handler>,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
enum Param {
    UserId,
    ProductId,
}

enum Handler {
    User,
    Product,
}

fn user_handler(id: &str) {
    println!("user: {}", id);
}

fn product_handler(id: &str) {
    println!("product: {}", id);
}

impl Router {
    fn new() -> Self {
        let mut builder = PrefixTreeMapBuilder::new();

        let user_url = "/user/:user_id/home";
        let product_url = "/product/:product_id/info";

        builder.insert(
            user_url.split('/').map(|part| {
                if part == ":user_id" {
                    KeyPart::Wildcard(Param::UserId)
                } else {
                    KeyPart::Exact(part)
                }
            }),
            Handler::User,
        );

        builder.insert(
            product_url.split('/').map(|part| {
                if part == ":product_id" {
                    KeyPart::Wildcard(Param::ProductId)
                } else {
                    KeyPart::Exact(part)
                }
            }),
            Handler::Product,
        );

        Self {
            table: builder.build(),
        }
    }

    fn route(&self, path: &str) {
        let key = path.split('/').collect::<Vec<_>>();

        let mut capture_map = Map::new();

        match self.table.find_and_capture(&key, &mut capture_map) {
            Some(Handler::User) => user_handler(capture_map.captures[0].as_ref().unwrap()),
            Some(Handler::Product) => product_handler(capture_map.captures[1].as_ref().unwrap()),
            None => println!("not found"),
        }
    }
}

struct Map {
    pub captures: [Option<String>; 2],
}

impl Map {
    fn new() -> Self {
        Self {
            captures: [None, None],
        }
    }
}

impl CaptureMap<Param, &str> for Map {
    fn insert(&mut self, key: Param, value: &str) {
        match key {
            Param::UserId => self.captures[0] = Some(value.to_string()),
            Param::ProductId => self.captures[1] = Some(value.to_string()),
        }
    }
}

fn main() {
    let router = Router::new();

    router.route("/user/00000/home");
    router.route("/user/0123456/home");
    router.route("/product/1234/info");
}
