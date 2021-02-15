//! account: bob, 100000, 0, validator

//! new-transaction
//! sender: bob
script {
use 0x1::VDF;
use 0x1::LibraAccount;
use 0x1::MinerState;
use 0x1::TestFixtures;
use 0x1::Signer;


// Test Prefix: 1301
fun main(sender: &signer) {
  // // Scenario: Bob, an existing validator, is sending a transaction for Eve, with a challenge and proof not yet submitted to the chain.
  let challenge = TestFixtures::eve_0_easy_chal();
  let solution = TestFixtures::eve_0_easy_sol();
  // // Parse key and check
  let (eve_addr, _auth_key) = VDF::extract_address_from_challenge(&challenge);
  assert(eve_addr == 0x3DC18D1CF61FAAC6AC70E3A63F062E4B, 401);
  
  let sender_addr = Signer::address_of(sender);
  let epochs_since_creation = 10;
  MinerState::test_helper_set_rate_limit(sender_addr, epochs_since_creation);

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
  );
}
}
// check: EXECUTED


//! new-transaction
//! sender: libraroot
script {
use 0x1::Reconfigure;
use 0x1::LibraAccount;
use 0x1::GAS::GAS;
use 0x1::Subsidy;
use 0x1::ValidatorUniverse;

fun main(vm: &signer) {
  let eve_addr = 0x3DC18D1CF61FAAC6AC70E3A63F062E4B;
  /// set the fullnode proof price to 0, to check if onboarding subsidy is given.
  Subsidy::test_set_fullnode_fixtures(vm, 0, 0, 0, 0, 0);
  Reconfigure::reconfigure(vm, 10); // need to remove testnet for this test, since testnet does not ratelimit account creation.
  let bal = LibraAccount::balance<GAS>(eve_addr);
  assert(bal == 576000, 73570001);

  assert(!ValidatorUniverse::test_exists_jailedbit(eve_addr), 7357001);
  assert(!ValidatorUniverse::is_in_universe(eve_addr), 7357002);
  assert(!ValidatorUniverse::is_jailed(eve_addr), 7357003);
}
}



//! new-transaction
//! sender: libraroot
script {
use 0x1::MinerState;
fun main(vm: &signer) {
  let eve_addr = 0x3DC18D1CF61FAAC6AC70E3A63F062E4B;
  /// mock mining above threshold.
  MinerState::test_helper_mock_mining_vm(vm, eve_addr, 100);
}
}

/// Validator is done sycning the node, and sends join transaction.



//! new-transaction
//! sender: libraroot
script {
use 0x1::MinerState;
use 0x1::LibraAccount;
use 0x1::ValidatorUniverse;

fun main(vm: &signer) {
  // simulate join validator set transaction
  let eve_addr = 0x3DC18D1CF61FAAC6AC70E3A63F062E4B;
  // let addr = Signer::address_of(validator);
  // if is above threshold continue, or raise error.
  let new_signer = LibraAccount::test_helper_create_signer(vm, eve_addr);
  assert(MinerState::node_above_thresh(&new_signer, eve_addr), 01);
  // if is not in universe, add back
  if (!ValidatorUniverse::is_in_universe(eve_addr)) {
      ValidatorUniverse::add_self(&new_signer);
  };
  // if is jailed, try to unjail
  if (ValidatorUniverse::is_jailed(eve_addr)) {
      ValidatorUniverse::unjail_self(&new_signer);
  };

  LibraAccount::test_helper_destroy_signer(vm, new_signer);
}
}


//! new-transaction
//! sender: libraroot
script {
use 0x1::ValidatorUniverse;
// use 0x1::Debug::print;

fun main(_vm: &signer) {
  let eve_addr = 0x3DC18D1CF61FAAC6AC70E3A63F062E4B;
  /// set the fullnode proof price to 0, to check if onboarding subsidy is given.
  /// mock mining above threshold.
  assert(ValidatorUniverse::test_exists_jailedbit(eve_addr), 7357001);
  assert(ValidatorUniverse::is_in_universe(eve_addr), 7357002);
  assert(!ValidatorUniverse::is_jailed(eve_addr), 7357003);
}
}