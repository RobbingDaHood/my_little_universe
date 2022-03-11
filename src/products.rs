
#[derive(Clone, PartialEq, Debug)]
pub struct Prices {
    min: i32,
    max: i32,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Product {
    Ores,
    Metals,
    PowerCells
}

pub fn get_prices(product: Product) -> Prices {
    match product {
        Product::Ores => { Prices { min: 5, max: 20 } }
        Product::Metals => { Prices { min: 100, max: 200 } }
        Product::PowerCells => { Prices { min: 1, max: 15 } }
    }
}


#[cfg(test)]
mod tests_int {
    use crate::products::get_prices;
    use crate::products::Product::{Metals, Ores};

    #[test]
    fn it_works() {
        assert_eq!(5, get_prices(Ores).min);
        assert_eq!(20, get_prices(Ores).max);
        assert_eq!(100, get_prices(Metals).min);
        assert_eq!(200, get_prices(Metals).max);
    }
}