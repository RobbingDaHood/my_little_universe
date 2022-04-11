use serde::{Deserialize, Serialize};

use crate::products::Product;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Eq, Hash)]
pub struct Amount {
    product: Product,
    amount: u32,
}

impl Amount {
    pub fn new(product: Product, amount: u32) -> Self {
        Self { product, amount }
    }

    pub fn product(&self) -> &Product {
        &self.product
    }
    pub fn amount(&self) -> u32 {
        self.amount
    }
}
