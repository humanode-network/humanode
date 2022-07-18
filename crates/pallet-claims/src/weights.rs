use frame_support::dispatch::Weight;

pub trait WeightInfo {
    fn claim() -> Weight;
    fn mint_claim() -> Weight;
    fn move_claim() -> Weight;
}

pub struct TestWeightInfo;
impl WeightInfo for TestWeightInfo {
    fn claim() -> Weight {
        0
    }
    fn mint_claim() -> Weight {
        0
    }
    fn move_claim() -> Weight {
        0
    }
}

// TODO: generate some deafult weights
impl WeightInfo for () {
    fn claim() -> Weight {
        0
    }
    fn mint_claim() -> Weight {
        0
    }
    fn move_claim() -> Weight {
        0
    }
}
