// Module to test bulk validator updates function in LibraSystem.move
//! account: alice, 1000000, 0, validator

//! new-transaction
//! sender: libraroot
script {
  use 0x1::MinerState;

  fun main() {
    let epochs_since_creation = 2; // below threshold
    MinerState::test_helper_set_rate_limit({{alice}}, epochs_since_creation);
    assert(MinerState::can_create_val({{alice}}) == false, 7357130101001000);
  }
}
//check: EXECUTED

//! new-transaction
//! sender: alice
script {
  use 0x1::LibraAccount;
  use 0x1::TestFixtures;
  use 0x1::Vector;

  fun main(sender: &signer) {
  let challenge = TestFixtures::eve_0_easy_chal();
  let solution = TestFixtures::eve_0_easy_sol();

  // intialize Eve, SHOULD ABORT
  LibraAccount::create_validator_account_with_proof(
      sender,
      &challenge,
      &solution,
      b"leet",
      0xfa72817f1b5aab94658238ddcdc08010,
      x"fa72817f1b5aab94658238ddcdc08010",
      x"8108aedfacf5cf1d73c67b6936397ba5fa72817f1b5aab94658238ddcdc08010", // random consensus_pubkey: vector<u8>,
      b"192.168.0.1", // validator_network_addresses: vector<u8>,
      b"192.168.0.1", // fullnode_network_addresses: vector<u8>,
      x"1ee7", // human_name: vector<u8>,
      Vector::singleton<address>({{alice}}),
      Vector::singleton<address>({{alice}}),
  );
  }
}
//check: ABORTED