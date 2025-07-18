use bytes::Bytes;
use ckb_contract_minitest::cells::sudt_data::{SUDTData, SUDTDataCell};
use ckb_contract_minitest::prelude::ContextExt;
use ckb_contract_minitest::ContractUtil;
use ckb_std::ckb_types::core::ScriptHashType;
use ckb_std::ckb_types::packed::{CellInput, CellOutput};
use ckb_std::ckb_types::prelude::{Builder, Entity, Pack, Unpack};
use ckb_testtool::ckb_types::core::TransactionBuilder;

// 0 -> 1
#[test]
fn test_mint_by_owner_0_to_1() {
    let mut ct = ContractUtil::new();
    let sudt_type_contract = ct.deploy_contract("sudt");

    // 1. Build the owner lock script using always_success with empty args
    let owner_lock_script = ct
        .context
        .build_script_with_hash_type(
            &ct.alway_contract,
            ScriptHashType::Data2,
            Bytes::new(), // empty args
        )
        .unwrap();
    let owner_hash: [u8; 32] = owner_lock_script.calc_script_hash().unpack();

    // 2. Create a simple owner cell as input (no type script, no data required)
    let owner_input_out_point = ct.context.create_cell(
        CellOutput::new_builder()
            .lock(owner_lock_script)
            .capacity((100 * 100_000_000u64).pack())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(owner_input_out_point)
        .build();

    // 3. Create SUDT output cell (type_args = owner_hash, amount = 1000)
    let sudt_output = SUDTDataCell::new(owner_hash, SUDTData { amount: 1000 });

    // 4. Construct the transaction with input and SUDT output
    let mut tx = TransactionBuilder::default().input(input).build();
    tx = ct.add_outpoint(
        tx,
        ct.alway_contract.clone(), // lock_contract for output (always_success)
        Some(sudt_type_contract.clone()), // type_contract for output (sudt)
        &sudt_output,
        100, // cell capacity for output
    );

    // 5. Complete the transaction by collecting required cell_deps and verify
    tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_passed(&tx, 1_000_000);
    println!("owner mint 0->1 ret:{:?}", ret);
}

// 0 -> N
#[test]
fn test_mint_by_owner_0_to_n() {
    let mut ct = ContractUtil::new();
    let sudt_type_contract = ct.deploy_contract("sudt");

    let owner_lock_script = ct
        .context
        .build_script_with_hash_type(
            &ct.alway_contract,
            ScriptHashType::Data2,
            Bytes::new(), // empty args
        )
        .unwrap();
    let owner_hash: [u8; 32] = owner_lock_script.calc_script_hash().unpack();

    let owner_input_out_point = ct.context.create_cell(
        CellOutput::new_builder()
            .lock(owner_lock_script)
            .capacity((100 * 100_000_000u64).pack())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(owner_input_out_point)
        .build();

    let output_count = 3;
    let mut outputs = Vec::new();
    for i in 0..output_count {
        // Each output cell gets an amount (for example, 1000 + i * 100)
        outputs.push(SUDTDataCell::new(
            owner_hash,
            SUDTData {
                amount: 1000 + i * 100,
            },
        ));
    }

    let mut tx = TransactionBuilder::default().input(input).build();
    for sudt_output in outputs.iter() {
        tx = ct.add_outpoint(
            tx,
            ct.alway_contract.clone(),
            Some(sudt_type_contract.clone()),
            sudt_output,
            100, // cell capacity for output
        );
    }

    tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_passed(&tx, 1_000_000);
    println!("owner mint 0->N ret:{:?}", ret);
}

// 1 -> 1
#[test]
fn test_sudt_transfer_1_to_1() {
    use ckb_contract_minitest::cells::sudt_data::{SUDTData, SUDTDataCell};
    use ckb_contract_minitest::ContractUtil;
    use ckb_testtool::ckb_types::core::TransactionBuilder;

    let mut ct = ContractUtil::new();
    let sudt_type_contract = ct.deploy_contract("sudt");

    let input_sudt_cell = SUDTDataCell::new([1; 32], SUDTData { amount: 1234 });
    let output_sudt_cell = SUDTDataCell::new([1; 32], SUDTData { amount: 1234 });

    let mut tx = TransactionBuilder::default().build();

    tx = ct.add_input(
        tx,
        ct.alway_contract.clone(),
        Some(sudt_type_contract.clone()),
        &input_sudt_cell,
        100,
    );

    tx = ct.add_outpoint(
        tx,
        ct.alway_contract.clone(),
        Some(sudt_type_contract.clone()),
        &output_sudt_cell,
        100,
    );

    tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_passed(&tx, 1_000_000);
    println!("transfer 1->1 ret:{:?}", ret);
}

// 1 -> N
#[test]
fn test_sudt_transfer_1_to_n() {
    let mut ct = ContractUtil::new();
    let sudt_type_contract = ct.deploy_contract("sudt");

    let total_amount = 3000;
    let input_sudt_cell = SUDTDataCell::new([1; 32], SUDTData { amount: total_amount });

    // 2. Prepare N output SUDT cells (e.g. split 3000 into 3 outputs: 1000, 1000, 1000)
    let output_amounts = vec![1000, 1000, 1000];
    let mut output_cells = Vec::new();
    for &amt in output_amounts.iter() {
        output_cells.push(SUDTDataCell::new([1; 32], SUDTData { amount: amt }));
    }

    let mut tx = TransactionBuilder::default().build();
    tx = ct.add_input(
        tx,
        ct.alway_contract.clone(),
        Some(sudt_type_contract.clone()),
        &input_sudt_cell,
        100,
    );
    for output_cell in output_cells.iter() {
        tx = ct.add_outpoint(
            tx,
            ct.alway_contract.clone(),
            Some(sudt_type_contract.clone()),
            output_cell,
            100,
        );
    }

    tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_passed(&tx, 1_000_000);
    println!("transfer 1->N ret:{:?}", ret);
}

// N -> 0
#[test]
fn test_sudt_burn_n_to_0_by_owner() {
let mut ct = ContractUtil::new();
    let sudt_type_contract = ct.deploy_contract("sudt");

    let owner_lock_script = ct.context.build_script_with_hash_type(
        &ct.alway_contract,
        ScriptHashType::Data2,
        Bytes::new(),
    ).unwrap();
    let owner_hash: [u8; 32] = owner_lock_script.calc_script_hash().unpack();

    // (1) Create the "owner cell" (lock_hash == owner_hash, empty data)
    let owner_input_out_point = ct.context.create_cell(
        CellOutput::new_builder()
            .lock(owner_lock_script.clone())
            .capacity((61 * 100_000_000u64).pack())
            .build(),
        Bytes::new(), // owner cell can have empty data
    );
    let owner_input = CellInput::new_builder()
        .previous_output(owner_input_out_point)
        .build();

    // (2) Create other SUDT cells (lock = always_success, type_args = owner_hash, data = 1000)
    let n = 3;
    let amount: u128 = 1000;
    let mut inputs = vec![owner_input];
    for _ in 1..n {
        // Build the output cell structure for SUDT input
        let sudt_output = CellOutput::new_builder()
            .lock(
                ct.context.build_script_with_hash_type(
                    &ct.alway_contract,
                    ScriptHashType::Data2,
                    Bytes::new(),
                ).unwrap()
            )
            .type_(
                Some(
                    ct.context.build_script_with_hash_type(
                        &sudt_type_contract,
                        ScriptHashType::Data2,
                        Bytes::from(owner_hash.to_vec()),
                    ).unwrap()
                ).pack()
            )
            .capacity((61 * 100_000_000u64).pack())
            .build();

        let sudt_data = amount.to_le_bytes().to_vec();

        let sudt_out_point = ct.context.create_cell(
            sudt_output,
            sudt_data.into(),
        );
        let input = CellInput::new_builder()
            .previous_output(sudt_out_point)
            .build();
        inputs.push(input);
    }

    // Construct tx: add all inputs, no outputs
    let mut tx = TransactionBuilder::default().build();
    for input in inputs {
        tx = tx.as_advanced_builder().input(input).build();
    }

    tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_passed(&tx, 1_000_000);
    println!("burn N->0 by owner ret:{:?}", ret);
}

// 1 -> 0
#[test]
fn test_sudt_burn_1_to_0_by_owner() {
    let mut ct = ContractUtil::new();
    let sudt_type_contract = ct.deploy_contract("sudt");

    let owner_lock_script = ct.context.build_script_with_hash_type(
        &ct.alway_contract,
        ScriptHashType::Data2,
        Bytes::new(),
    ).unwrap();
    let owner_hash: [u8; 32] = owner_lock_script.calc_script_hash().unpack();

    let sudt_output = CellOutput::new_builder()
        .lock(
            ct.context.build_script_with_hash_type(
                &ct.alway_contract,
                ScriptHashType::Data2,
                Bytes::new(),
            ).unwrap()
        )
        .type_(
            Some(
                ct.context.build_script_with_hash_type(
                    &sudt_type_contract,
                    ScriptHashType::Data2,
                    Bytes::from(owner_hash.to_vec()),
                ).unwrap()
            ).pack()
        )
        .capacity((61 * 100_000_000u64).pack())
        .build();

    let amount: u128 = 1000;
    let sudt_data = amount.to_le_bytes().to_vec();

    let sudt_out_point = ct.context.create_cell(
        sudt_output,
        sudt_data.into(),
    );
    let sudt_input = CellInput::new_builder()
        .previous_output(sudt_out_point)
        .build();

    let owner_input_out_point = ct.context.create_cell(
        CellOutput::new_builder()
            .lock(owner_lock_script.clone())
            .capacity((61 * 100_000_000u64).pack())
            .build(),
        Bytes::new(), // owner cell can have empty data
    );
    let owner_input = CellInput::new_builder()
        .previous_output(owner_input_out_point)
        .build();

    let mut tx = TransactionBuilder::default().build();
    tx = tx.as_advanced_builder().input(sudt_input).input(owner_input).build();

    tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_passed(&tx, 1_000_000);
    println!("burn 1->0 by owner ret:{:?}", ret);
}

// N -> 1
#[test]
fn test_sudt_transfer_n_to_1() {
    use ckb_contract_minitest::cells::sudt_data::{SUDTData, SUDTDataCell};
    use ckb_contract_minitest::ContractUtil;
    use ckb_testtool::ckb_types::core::TransactionBuilder;

    let mut ct = ContractUtil::new();
    let sudt_type_contract = ct.deploy_contract("sudt");

    // Prepare N input SUDT cells (e.g., 3 inputs: 500, 800, 700 = total 2000)
    let input_amounts = vec![500, 800, 700];
    let mut input_cells = Vec::new();
    for &amt in input_amounts.iter() {
        input_cells.push(SUDTDataCell::new([1; 32], SUDTData { amount: amt }));
    }

    // Prepare 1 output SUDT cell (total amount)
    let total_amount: u128 = input_amounts.iter().sum();
    let output_cell = SUDTDataCell::new([1; 32], SUDTData { amount: total_amount });

    let mut tx = TransactionBuilder::default().build();
    for input_cell in input_cells.iter() {
        tx = ct.add_input(
            tx,
            ct.alway_contract.clone(),
            Some(sudt_type_contract.clone()),
            input_cell,
            100,
        );
    }
    tx = ct.add_outpoint(
        tx,
        ct.alway_contract.clone(),
        Some(sudt_type_contract.clone()),
        &output_cell,
        100,
    );

    tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_passed(&tx, 1_000_000);
    println!("transfer N->1 ret:{:?}", ret);
}

// N -> N
#[test]
fn test_sudt_transfer_n_to_n() {
    use ckb_contract_minitest::cells::sudt_data::{SUDTData, SUDTDataCell};
    use ckb_contract_minitest::ContractUtil;
    use ckb_testtool::ckb_types::core::TransactionBuilder;

    let mut ct = ContractUtil::new();
    let sudt_type_contract = ct.deploy_contract("sudt");

    // Prepare M input SUDT cells
    let input_amounts = vec![1500, 800, 700]; // total 3000
    let mut input_cells = Vec::new();
    for &amt in input_amounts.iter() {
        input_cells.push(SUDTDataCell::new([1; 32], SUDTData { amount: amt }));
    }

    // Prepare N output SUDT cells (split 3000 into 2 outputs: 1200, 1800)
    let output_amounts = vec![1200, 1800];
    let mut output_cells = Vec::new();
    for &amt in output_amounts.iter() {
        output_cells.push(SUDTDataCell::new([1; 32], SUDTData { amount: amt }));
    }

    let mut tx = TransactionBuilder::default().build();
    for input_cell in input_cells.iter() {
        tx = ct.add_input(
            tx,
            ct.alway_contract.clone(),
            Some(sudt_type_contract.clone()),
            input_cell,
            100,
        );
    }
    for output_cell in output_cells.iter() {
        tx = ct.add_outpoint(
            tx,
            ct.alway_contract.clone(),
            Some(sudt_type_contract.clone()),
            output_cell,
            100,
        );
    }

    tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_passed(&tx, 1_000_000);
    println!("transfer N->N ret:{:?}", ret);
}
