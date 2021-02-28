address 0x123 {
module BondingCurve {
  use 0x1::Signer;
  // use 0x1::Debug::print;

  resource struct CurveState {
    is_deprecated: bool,
    reserve: u64,
    supply: u64,
    kappa: u64
  }

  resource struct Token { 
    value: u64
  }

  fun sunset() {
    // if true state.is_deprecated == true
    // allow holders to redeem at the spot price at sunset.
    // cannot receive new deposits
    //TBD
  }

  ///////// Initialization /////////
  public fun initialize_curve(
    sponsor: &signer,
    reserve: u64,
    spot_price: u64,
    kappa: u64
  ) {
    // TODO: Kappa will be overriden with `2` until math natives.
    kappa = 2;

    assert(reserve > 0, 7357001);

    let supply = calc_supply_from_price(kappa, reserve, spot_price);

    let init_state = CurveState {
      is_deprecated: false, // deprecate mode
      reserve: reserve,
      supply: supply,
      kappa: kappa
    };


    // This initializes the contract, and stores the contract state at the address of sender. TDB where the state gets stored.
    move_to<CurveState>(sponsor, init_state);


    let first_token = Token { 
      value: supply
    };

    // minting the first coin, sponsor is recipent of initial coin.
    move_to<Token>(sponsor, first_token);
  }

  ///////// Calculations /////////
  fun curve_kappa_two(add_to_reserve: u64, supply: u64, reserve: u64):u64 {
    supply * sqrt(1+(add_to_reserve/reserve))
  }

  /// TODO: distant future, generalized kappa. Need math natives.
  fun curve_any_kappa() {

  }

  fun calc_supply_from_price(kappa: u64, reserve_balance: u64, spot_price: u64): u64 {
    kappa * (reserve_balance/spot_price)
  }

  fun calc_fee():u64 {
    1
  }

  fun deposit_token_to(sender: &signer, new_value: u64) acquires Token {
    let to_addr = Signer::address_of(sender);
    if (!exists<Token>(to_addr)) {
      move_to<Token>(sender, Token { value: new_value });
    } else {
      let user_token = borrow_global_mut<Token>(to_addr);
      user_token.value = user_token.value + new_value;
    }
  }

  fun withdraw_token_from(sender: &signer, sub_value: u64) acquires Token {
    let from_addr = Signer::address_of(sender);
    assert(exists<Token>(from_addr), 73570005);
    let user_token = borrow_global_mut<Token>(from_addr);
    user_token.value = user_token.value - sub_value;
  }

  ///////// API /////////
  public fun bond_to_mint(sender: &signer, sponsor_addr: address, add_to_reserve: u64):u64 acquires CurveState, Token{
    assert(exists<CurveState>(sponsor_addr), 73570002);
    let state = borrow_global_mut<CurveState>(sponsor_addr);

    let new_supply = 0;
    if (state.kappa == 2) {
      new_supply = curve_kappa_two(add_to_reserve, state.supply, state.reserve);
    };
    
    let mint = new_supply - state.supply;
    deposit_token_to(sender, mint);
    // new curve state
    state.reserve = state.reserve + add_to_reserve;
    state.supply = new_supply;
    mint
  }

  public fun burn_to_withdraw(_upgrade_token: u64):u64 {
    1
  }


  ///////// GETTERS /////////
  public fun get_curve_state(sponsor_address: address): (u64, u64) acquires CurveState {
    let state = borrow_global<CurveState>(sponsor_address);
    (state.reserve, state.supply)
  }

  public fun get_user_balance(addr: address): u64 acquires Token {
    let state = borrow_global<Token>(addr);
    state.value
  }

  ///////// MATH /////////

  //// !!!!! DANGER using while loop for square root for prototype !!!!! ////////
  fun sqrt(y: u64): u64 {
    if (y > 3) {
        let z = y;
        let x = y / 2 + 1;
        while (x < z) {
            z = x;
            x = (y / x + x) / 2;
        };
        return x
    } else if (y != 0) {
        return 1
    };
    0
  }

}
}