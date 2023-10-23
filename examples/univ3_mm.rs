use clap::Parser;
use ethers::providers::{Http, Middleware, Provider};
use ethers_core::types::{Address, BlockId, BlockNumber, H160, H256, U256, U64};
use halo2_base::gates::GateInstructions;
use halo2_base::safe_types::RangeInstructions;
use halo2_base::utils::{BigPrimeField, ScalarField};
use halo2_base::AssignedValue;
use halo2_base::Context;
use halo2_base::QuantumCell::Constant;
use halo2_scaffold::gadget::fixed_point::{FixedPointChip, FixedPointInstructions};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::{run, run_builder_on_inputs};
use num_bigint::BigUint;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use std::env::{set_var, var};

const N: usize = 64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub x: Vec<Vec<u8>>, // field element, but easier to deserialize as a string
}

// https://hackmd.io/@tazAymRSQCGXTUKkbh1BAg/r1vcQLl8is
fn sqrtx96_to_63fp63<F: ScalarField>(
    ctx: &mut Context<F>,
    sqrtx96: &AssignedValue<F>,
    fpc: &FixedPointChip<F, 63>,
) -> halo2_base::AssignedValue<F>
where
    F: BigPrimeField,
{
    let range_chip = fpc.range_gate();

    let (q, _) = range_chip.div_mod(ctx, *sqrtx96, 1_u128 << 32, 160);
    // println!("x: {:?}", *sqrtx96.value());
    // println!("q: {:?}", q.value());
    // println!("r: {:?}", r.value());

    let q_sq = range_chip.gate.mul(ctx, q, q);
    // println!("qq: {qq:?}");

    let bu = BigUint::from_u128(2).unwrap();
    let d_2_128 = BigUint::pow(&bu, 128);
    // println!("d_2_128: {:?}", d_2_128);
    let (q2, _) = range_chip.div_mod(ctx, q_sq, d_2_128, 160);
    let a = Constant(F::from_str_vartime("9223372036854775808").unwrap());
    let q3 = range_chip.gate.mul(ctx, q2, a);

    let ten2twelve = fpc.quantization(1000000000000.0);
    let q4 = fpc.qdiv(ctx, q3, Constant(ten2twelve));
    let d = fpc.dequantization(*q4.value());
    println!("{d:?},");
    q4
}

fn fp63_to_sqrtx96<F: ScalarField>(
    ctx: &mut Context<F>,
    fp63: &AssignedValue<F>,
    decimals: u32,
    fpc: &FixedPointChip<F, 63>,
) -> halo2_base::AssignedValue<F>
where
    F: BigPrimeField,
{
    let range_chip = fpc.range_gate();
    let a = Constant(F::from_u128(10_u128.pow(decimals)));

    let norm_fp63 = range_chip.gate().mul(ctx, *fp63, a);
    let b = Constant(F::from_u128(2_u128.pow(33)));
    range_chip.gate().mul(ctx, norm_fp63, b)
}

fn some_algorithm_in_zk<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) where
    F: BigPrimeField,
{
    const PRECISION_BITS: u32 = 63;
    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    let fpc = FixedPointChip::<F, PRECISION_BITS>::default(lookup_bits);

    let zero = ctx.load_constant(F::from_u128(0u128));
    // fixed-point exp arithmetic

    let qPrices = (0..N)
        .map(|i| {
            let x = F::from_bytes_le(&input.x[i]);
            let x = ctx.load_witness(x);
            make_public.push(x);
            sqrtx96_to_63fp63(ctx, &x, &fpc)
        })
        .collect::<Vec<AssignedValue<F>>>();

    let sum = qPrices.iter().fold(zero, |acc, x| fpc.qadd(ctx, acc, *x));
    let len = ctx.load_constant(fpc.quantization(N as f64));
    println!("Sum {:?}", fpc.dequantization(*sum.value()));
    println!("len {:?}", *len.value());
    let TWO: AssignedValue<F> = ctx.load_constant(F::from_u128(2_u128)); //ctx.load_constant(fpc.quantization(2.0));
    println!("TWO {:?}", *TWO.value());
    let mean: AssignedValue<F> = fpc.qdiv(ctx, sum, len);

    println!("Diff");
    let sq_diff = qPrices
        .into_iter()
        .map(|x| {
            let diff = fpc.qsub(ctx, x, mean);
            let diff_2 = fpc.qmul(ctx, diff, diff);
            println!("{:?}", fpc.dequantization(*diff_2.value()));
            diff_2
        })
        .collect::<Vec<AssignedValue<F>>>();
    let sq_diff_sum = sq_diff.iter().fold(zero, |acc, x| fpc.qadd(ctx, acc, *x));
    let avg_sq_diff_sum = fpc.qdiv(ctx, sq_diff_sum, len);
    let std = fpc.qsqrt(ctx, avg_sq_diff_sum);
    // println!("sq_diff_sum {:?}", fpc.dequantization(*sq_diff_sum.value()));
    println!("std {:?}", fpc.dequantization(*std.value()));
    let two_std = fpc.qadd(ctx, std, std);
    let lower_bound = fpc.qsub(ctx, mean, two_std);
    let upper_bound = fpc.qadd(ctx, mean, two_std);

    println!("mean {:?}", fpc.dequantization(*mean.value()));
    println!("lower_bound {:?}", fpc.dequantization(*lower_bound.value()));
    println!("upper_bound {:?}", fpc.dequantization(*upper_bound.value()));
    println!("mean_fp {:?}", *mean.value());
    println!("lower_bound_fp {:?}", *lower_bound.value());
    println!("upper_bound_fp {:?}", *upper_bound.value());

    //let sqrt_mean = fpc.qsqrt(ctx, mean);
    let sqrt_lower_bound = fpc.qsqrt(ctx, lower_bound);
    let sqrt_upper_bound = fpc.qsqrt(ctx, upper_bound);

    //println!("sqrt_mean {:?}", fpc.dequantization(*sqrt_mean.value()));
    println!("sqrt_lower_bound {:?}", fpc.dequantization(*sqrt_lower_bound.value()));
    println!("sqrt_upper_bound {:?}", fpc.dequantization(*sqrt_upper_bound.value()));
    //println!("sqrt_mean_fp {:?}", *sqrt_mean.value());
    println!("sqrt_lower_bound_fp {:?}", *sqrt_lower_bound.value());
    println!("sqrt_upper_bound_fp {:?}", *sqrt_upper_bound.value());

    // let sqrt96_mean = fp63_to_sqrtx96(ctx, &sqrt_mean, 6, &fpc);
    let sqrt96_lb = fp63_to_sqrtx96(ctx, &sqrt_lower_bound, 6, &fpc);
    let sqrt96_ub = fp63_to_sqrtx96(ctx, &sqrt_upper_bound, 6, &fpc);
    //println!("sqrt96_mean {:?}", *sqrt96_mean.value());
    println!("sqrt96_lb {:?}", *sqrt96_lb.value());
    println!("sqrt96_ub {:?}", *sqrt96_ub.value());

    make_public.push(sqrt96_lb);
    make_public.push(sqrt96_ub);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(
        //"https://eth-mainnet.g.alchemy.com/v2/B6N7JmrpWczBs5xEXCfk6on2xqrQ3EIq",
        "https://eth-goerli.g.alchemy.com/v2/q4W0PaEugB3k4NDVqEr8M1Qy87FAd8a3",
    )?;
    let start_block = 9852686;
    //let block_number: U64 = provider.get_block_number().await?;
    //let address = "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640".parse::<Address>()?; // MAINNET
    let address = "0x297FFb1BbAc2F906A7c8f10808E2E48825CF5b7f".parse::<Address>()?;
    let slot_number: H256 = H256::from_low_u64_be(0);

    let blocks_per_hour = 12 * 5 * 10;

    const U160_BYTES: usize = 20;
    // let mut prices: [u8; N * U160_BYTES];
    //let mut prices = [[0u8; U160_BYTES]; N];
    let mut prices: Vec<Vec<u8>> = Vec::new();
    for hour in 0..N {
        let block_number = BlockId::Number((start_block - hour * blocks_per_hour).into());
        let slot1 = provider.get_storage_at(address, slot_number, Some(block_number)).await?;
        let mut srqt_ratio_x96_bytes =
            <[u8; U160_BYTES]>::try_from(&slot1.as_bytes()[(32 - U160_BYTES)..])?;

        let srqt_ratio_x96 = H160::from(srqt_ratio_x96_bytes);
        println!("slot1: {slot1:?}");
        println!("srqt_ratio_x96: {srqt_ratio_x96:?}");

        srqt_ratio_x96_bytes.reverse();
        prices.push(Vec::from(srqt_ratio_x96_bytes));
        //prices[hour][0..].clone_from_slice(&srqt_ratio_x96_bytes);
    }

    // let slot1 = provider.get_storage_at(H256::from(1), BlockId::from(18_149_980)).await?;
    // println!("{block_number}");
    env_logger::init();
    // genrally lookup_bits is degree - 1
    // set_var("LOOKUP_BITS", 12.to_string());
    // set_var("DEGREE", 13.to_string());
    let args: Cli = Cli::parse();

    let input = CircuitInput { x: prices };

    run_builder_on_inputs(
        |builder, input, public| some_algorithm_in_zk(builder.main(0), input, public),
        args,
        input,
    );

    Ok(())
}
