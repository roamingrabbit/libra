//! Key generation
use std::env;

// use abscissa_core::{status_info, status_warn};
use libra_wallet::{Mnemonic, WalletLibrary};
use libra_types::{
  account_address::AccountAddress,
  transaction::authenticator::AuthenticationKey
};

/// Genereates keys from WalletLibrary, updates a MinerConfig
pub fn keygen() -> (AuthenticationKey, AccountAddress, WalletLibrary) {
        // Generate new keys
        let mut wallet = WalletLibrary::new();
        let mnemonic_string = wallet.mnemonic();
        // NOTE: Authkey uses the child number 0 by default
        let (auth_key, _) = wallet.new_address().expect("Could not generate address");
        let account = auth_key.derived_address();
        //////////////// Info ////////////////
        
        println!("0L Account Address:\n\
        ...........................\n\
        {:?}\n", &account);

        // TODO: Auth key is not needed anywhere.
        // println!("0L Auth Key:\n\
        // ...........................\n\
        // {:?}\n", &auth_key.to_string());

        println!("0L mnemonic:\n\
        ...........................");
      

        println!("{}\n", &mnemonic_string.as_str());
        println!("WRITE THIS DOWN NOW. This is the last time you will see \
                  this mnemonic. It is not saved anywhere. Nobody can help \
                  you if you lose it.\n\n");

        (auth_key, account, wallet)
}

/// Get authkey and account from mnemonic
pub fn get_account_from_mnem(mnemonic_string: String) 
  -> (AuthenticationKey, AccountAddress, WalletLibrary) {
      let mut wallet = WalletLibrary::new_from_mnemonic(
        Mnemonic::from(&mnemonic_string).unwrap()
      );
      let (auth_key, _) = wallet.new_address().expect("Could not generate address");
      let account = auth_key.derived_address();
      (auth_key, account, wallet)
}

/// Prompts user to type mnemonic securely.
pub fn account_from_prompt() 
  -> (AuthenticationKey, AccountAddress, WalletLibrary) {
    println!("Enter your 0L mnemonic:");
    // TODO: Simplify mnemonic entry for CI/debug cases

    match env::var("NODE_ENV") {
      Ok(val) => {
        let maybe_env_mnem = env::var("MNEM");
        
        // if we are in debugging or CI mode
        if val != "prod" && maybe_env_mnem.is_ok() {
          println!("Debugging mode, using mnemonic from env variable, $MNEM");
          return get_account_from_mnem(maybe_env_mnem.unwrap().trim().to_string())
        }
        println!("Debugging mode, you can set mnemonic to env $MNEM");
      },
      _ => {}, // default to "prod" if not set
    };
    
    let read = rpassword::read_password_from_tty(Some("\u{1F511} "));
    get_account_from_mnem(read.unwrap().trim().to_string())
}

#[test]
fn wallet() { 
    use libra_wallet::Mnemonic;
    let mut wallet = WalletLibrary::new();

    let (auth_key, child_number) = wallet.new_address().expect(
      "Could not generate address"
    );
    let mnemonic_string = wallet.mnemonic(); //wallet

    println!("auth_key:\n{:?}", auth_key.to_string());
    println!("child_number:\n{:?}", child_number);
    println!("mnemonic:\n{}", mnemonic_string);

    let mut wallet = WalletLibrary::new_from_mnemonic(
      Mnemonic::from(&mnemonic_string).unwrap()
    );

    // println!("wallet\n:{:?}", wallet);

    let (main_addr, child_number ) = wallet.new_address().unwrap();
    println!("wallet\n:{:?} === {:x}", child_number, main_addr);

    let vec_addresses = wallet.get_addresses().unwrap();

    println!("vec_addresses\n:{:?}", vec_addresses);

    // Expect this to be zero before we haven't populated the address map in the repo
    assert!(vec_addresses.len() == 1);
    // Empty hashmap should be fine
    // let mut vec_account_data = Vec::new();
}