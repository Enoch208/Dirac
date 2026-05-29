fn main() {
    sails_rs::ClientBuilder::<colosseum::ColosseumProgram>::from_env()
        .with_idl_path("../colosseum/colosseum.idl")
        .build_idl()
        .with_mocks("with_mocks")
        .generate()
        .unwrap();
}
