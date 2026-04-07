use noesis_data::repositories::biofield_repository::BiofieldRepository;

#[test]
fn biofield_repository_contract_is_exposed() {
    let _ = std::mem::size_of::<BiofieldRepository>();
}
