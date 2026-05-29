fn main() {
    sails_rs::ClientBuilder::<dirac::DiracProgram>::from_env()
        .with_idl_path("../dirac/dirac.idl")
        .build_idl()
        .with_mocks("with_mocks")
        .generate()
        .unwrap();
}
