macro_rules! try_or_fail {
	( $e:expr ) => {
		match $e {
			Ok(v) => v,
			Err(e) => return Control::Exit(e.into()),
		}
	};
}

macro_rules! pop {
	( $machine:expr, $( $x:ident ),* ) => (
		$(
			let $x = match $machine.stack.pop() {
				Ok(value) => value,
				Err(e) => return Control::Exit(e.into()),
			};
		)*
	);
}

macro_rules! pop_u256 {
	( $machine:expr, $( $x:ident ),* ) => (
		$(
			let $x = match $machine.stack.pop() {
				Ok(value) => U256::from_big_endian(&value[..]),
				Err(e) => return Control::Exit(e.into()),
			};
		)*
	);
}

macro_rules! push {
	( $machine:expr, $( $x:expr ),* ) => (
		$(
			match $machine.stack.push($x) {
				Ok(()) => (),
				Err(e) => return Control::Exit(e.into()),
			}
		)*
	)
}

macro_rules! push_u256 {
	( $machine:expr, $( $x:expr ),* ) => (
		$(
			let mut value = H256::default();
			$x.to_big_endian(&mut value[..]);
			match $machine.stack.push(value) {
				Ok(()) => (),
				Err(e) => return Control::Exit(e.into()),
			}
		)*
	)
}

macro_rules! op1_u256_fn {
	( $machine:expr, $op:path ) => {{
		pop_u256!($machine, op1);
		let ret = $op(op1);
		push_u256!($machine, ret);

		Control::Continue(1)
	}};
}

macro_rules! op2_u256_bool_ref {
	( $machine:expr, $op:ident ) => {{
		pop_u256!($machine, op1, op2);
		let ret = op1.$op(&op2);
		push_u256!($machine, if ret { U256::one() } else { U256::zero() });

		Control::Continue(1)
	}};
}

macro_rules! op2_u256 {
	( $machine:expr, $op:ident ) => {{
		pop_u256!($machine, op1, op2);
		let ret = op1.$op(op2);
		push_u256!($machine, ret);

		Control::Continue(1)
	}};
}

macro_rules! op2_u256_tuple {
	( $machine:expr, $op:ident ) => {{
		pop_u256!($machine, op1, op2);
		let (ret, ..) = op1.$op(op2);
		push_u256!($machine, ret);

		Control::Continue(1)
	}};
}

macro_rules! op2_u256_fn {
	( $machine:expr, $op:path ) => {{
		pop_u256!($machine, op1, op2);
		let ret = $op(op1, op2);
		push_u256!($machine, ret);

		Control::Continue(1)
	}};
}

macro_rules! op3_u256_fn {
	( $machine:expr, $op:path ) => {{
		pop_u256!($machine, op1, op2, op3);
		let ret = $op(op1, op2, op3);
		push_u256!($machine, ret);

		Control::Continue(1)
	}};
}

macro_rules! as_usize_or_fail {
	( $v:expr ) => {{
		if $v > U256::from(usize::MAX) {
			return Control::Exit(ExitFatal::NotSupported.into());
		}

		$v.as_usize()
	}};

	( $v:expr, $reason:expr ) => {{
		if $v > U256::from(usize::MAX) {
			return Control::Exit($reason.into());
		}

		$v.as_usize()
	}};
}
