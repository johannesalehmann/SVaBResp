mod benchmark;

use prism_model_builder::ModelBuildingOutput;
use prism_model_builder::probabilistic_models::{MdpType, ModelTypes};
use std::time::Duration;

fn main() {
    BenchmarkResult::print_header();
    let too_long_time = 1_000_000;
    let models = vec![
        (
            32,
            "consensus(N=4,K=2)",
            "mdp/consensus/consensus.4.prism",
            "N=4;K=2",
            22656,
            75232,
        ),
        (
            2929,
            "consensus(N=6,K=2)",
            "mdp/consensus/consensus.6.prism",
            "N=6;K=2",
            1258240,
            6236736,
        ),
        (
            too_long_time,
            "consensus(N=8,K=2)",
            "mdp/consensus/consensus.8.prism",
            "N=8;K=2",
            61018112,
            403856384,
        ),
        (
            1,
            "csma(N=2,K=2)",
            "mdp/csma/csma.2-2.prism",
            "",
            1038,
            1282,
        ),
        (
            14,
            "csma(N=2,K=4)",
            "mdp/csma/csma.2-4.prism",
            "",
            7958,
            10594,
        ),
        (
            133,
            "csma(N=2,K=6)",
            "mdp/csma/csma.2-6.prism",
            "",
            66718,
            93072,
        ),
        (
            86,
            "csma(N=3,K=2)",
            "mdp/csma/csma.3-2.prism",
            "",
            36850,
            55862,
        ),
        (
            3982,
            "csma(N=3,K=4)",
            "mdp/csma/csma.3-4.prism",
            "",
            1460287,
            2396727,
        ),
        (
            too_long_time,
            "csma(N=3,K=6)",
            "mdp/csma/csma.3-6.prism",
            "",
            84856004,
            147200984,
        ),
        (
            2558,
            "csma(N=4,K=2)",
            "mdp/csma/csma.4-2.prism",
            "",
            761962,
            1327068,
        ),
        (
            too_long_time,
            "csma(N=4,K=4)",
            "mdp/csma/csma.4-4.prism",
            "",
            133301572,
            258474290,
        ),
        (
            17,
            "eajs(2,100,5)",
            "mdp/eajs/eajs.2.prism",
            "energy_capacity=100;B=5",
            12828,
            21795,
        ),
        (
            334,
            "eajs(3,150,7)",
            "mdp/eajs/eajs.3.prism",
            "energy_capacity=150;B=7",
            143155,
            274496,
        ),
        (
            2859,
            "eajs(4,200,9)",
            "mdp/eajs/eajs.4.prism",
            "energy_capacity=200;B=9",
            872410,
            1806871,
        ),
        (
            13617,
            "eajs(5,250,1)",
            "mdp/eajs/eajs.5.prism",
            "energy_capacity=250;B=11",
            3049471,
            6977654,
        ),
        (
            48963,
            "eajs(6,300,13)",
            "mdp/eajs/eajs.6.prism",
            "energy_capacity=300;B=13",
            7901694,
            19722777,
        ),
        (
            14,
            "firewire(false,3,0)",
            "mdp/firewire/firewire.false.prism",
            "delay=3;deadline=0",
            4093,
            5585,
        ),
        (
            774,
            "firewire(false,36,0)",
            "mdp/firewire/firewire.false.prism",
            "delay=36;deadline=0",
            212268,
            481792,
        ),
        (
            298,
            "firewire(true,3,200)",
            "mdp/firewire/firewire.true.prism",
            "delay=3;deadline=200",
            83153,
            115467,
        ),
        (
            1561,
            "firewire(true,3,400)",
            "mdp/firewire/firewire.true.prism",
            "delay=3;deadline=400",
            430537,
            586224,
        ),
        (
            4055,
            "firewire(true,3,600)",
            "mdp/firewire/firewire.true.prism",
            "delay=3;deadline=600",
            1078695,
            1476198,
        ),
        (
            7189,
            "firewire(true,3,800)",
            "mdp/firewire/firewire.true.prism",
            "delay=3;deadline=800",
            1887528,
            2576406,
        ),
        (
            28379,
            "firewire(true,36,200)",
            "mdp/firewire/firewire.true.prism",
            "delay=36;deadline=200",
            6932041,
            15783496,
        ),
        (
            too_long_time,
            "firewire(true,36,400)",
            "mdp/firewire/firewire.true.prism",
            "delay=36;deadline=400",
            44578503,
            101410391,
        ),
        (
            too_long_time,
            "firewire(true,36,600)",
            "mdp/firewire/firewire.true.prism",
            "delay=36;deadline=600",
            87025747, // Values need updating
            197753727,
        ),
        (
            too_long_time,
            "firewire(true,36,800)",
            "mdp/firewire/firewire.true.prism",
            "delay=36;deadline=800",
            129267079, // Values need updating
            293635687,
        ),
        (6519, "ij(20)", "mdp/ij/ij.20.prism", "", 1048575, 18350060),
        (
            3456,
            "pnueli-zuck(5)",
            "mdp/pnueli-zuck/pnueli-zuck.5.prism",
            "",
            397435,
            2492035,
        ),
        (39, "rabin(3)", "mdp/rabin/rabin.3.prism", "", 27766, 137802),
        (
            too_long_time,
            "rabin(5)",
            "mdp/rabin/rabin.5.prism",
            "",
            27381358,
            177834300,
        ),
        (8, "wlan(0,0)", "mdp/wlan/wlan.0.prism", "COL=0", 2954, 5202),
        (
            27,
            "wlan(1,0)",
            "mdp/wlan/wlan.1.prism",
            "COL=0",
            8625,
            16196,
        ),
        (
            92,
            "wlan(2,0)",
            "mdp/wlan/wlan.2.prism",
            "COL=0",
            28480,
            57164,
        ),
        (
            330,
            "wlan(3,0)",
            "mdp/wlan/wlan.3.prism",
            "COL=0",
            96302,
            204576,
        ),
        (
            1261,
            "wlan(4,0)",
            "mdp/wlan/wlan.4.prism",
            "COL=0",
            345000,
            762252,
        ),
        (
            5081,
            "wlan(5,0)",
            "mdp/wlan/wlan.5.prism",
            "COL=0",
            1295218,
            2929960,
        ),
        (
            21586,
            "wlan(6,0)",
            "mdp/wlan/wlan.6.prism",
            "COL=0",
            5007548,
            11475748,
        ),
        (
            3,
            "zeroconf(6,0)",
            "mdp/zeroconf/zeroconf.prism",
            "N=1000;K=8;reset=true",
            1924,
            2845,
        ),
        (
            8,
            "brp(64,5)",
            "dtmc/brp/brp.prism",
            "N=64;MAX=5",
            5192,
            6915,
        ),
        (
            10,
            "crowds(3,10)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=3;CrowdSize=10",
            6563,
            15143,
        ),
        (
            35,
            "crowds(3,15)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=3;CrowdSize=15",
            19228,
            55948,
        ),
        (
            87,
            "crowds(3,20)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=3;CrowdSize=20",
            42318,
            148578,
        ),
        (
            53,
            "crowds(4,10)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=4;CrowdSize=10",
            30070,
            70110,
        ),
        (
            249,
            "crowds(4,15)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=4;CrowdSize=15",
            119800,
            352360,
        ),
        (
            723,
            "crowds(4,20)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=4,CrowdSize=20",
            333455,
            1183535,
        ),
        (
            194,
            "crowds(5,10)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=5,CrowdSize=10",
            111294,
            261444,
        ),
        (
            1228,
            "crowds(5,15)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=5,CrowdSize=15",
            592060,
            1754860,
        ),
        (
            4816,
            "crowds(5,20)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=5,CrowdSize=20",
            2061951,
            7374951,
        ),
        (
            30,
            "crowds(6,5)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=6,CrowdSize=5",
            18817,
            32677,
        ),
        (
            635,
            "crowds(6,10)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=6,CrowdSize=10",
            352535,
            833015,
        ),
        (
            5154,
            "crowds(6,15)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=6,CrowdSize=15",
            2464168,
            7347928,
        ),
        (
            65910,
            "crowds(6,20)",
            "dtmc/crowds/crowds.prism",
            "TotalRuns=6,CrowdSize=20",
            10633591,
            38261191,
        ),
        (
            147,
            "egl(5,2)",
            "dtmc/egl/egl.prism",
            "N=5,L=2",
            33790,
            34813,
        ),
        (
            316,
            "egl(5,4)",
            "dtmc/egl/egl.prism",
            "N=5,L=4",
            74750,
            75773,
        ),
        (
            510,
            "egl(5,6)",
            "dtmc/egl/egl.prism",
            "N=5,L=6",
            115710,
            116733,
        ),
        (
            668,
            "egl(5,8)",
            "dtmc/egl/egl.prism",
            "N=5,L=8",
            156670,
            157693,
        ),
        (
            too_long_time,
            "egl(10,2)",
            "dtmc/egl/egl.prism",
            "N=10,L=2",
            66060286,
            67108861,
        ),
        // Herman currently not supported because of init constraints
        // (
        //     true,
        //     "herman(7)",
        //     "dtmc/herman/herman.7.prism",
        //     "",
        //     128,
        //     2188,
        // ),
        // (
        //     true,
        //     "herman(9)",
        //     "dtmc/herman/herman.9.prism",
        //     "",
        //     512,
        //     19684,
        // ),
        // (
        //     true,
        //     "herman(11)",
        //     "dtmc/herman/herman.11.prism",
        //     "",
        //     2048,
        //     177148,
        // ),
        // (
        //     true,
        //     "herman(13)",
        //     "dtmc/herman/herman.13.prism",
        //     "",
        //     8192,
        //     1594324,
        // ),
        // (
        //     true,
        //     "herman(15)",
        //     "dtmc/herman/herman.15.prism",
        //     "",
        //     32768,
        //     14348908,
        // ),
        // (
        //     false,
        //     "herman(17)",
        //     "dtmc/herman/herman.17.prism",
        //     "",
        //     131072,
        //     129140164,
        // ),
        (
            10,
            "leader_sync(5,4)",
            "dtmc/leader_sync/leader_sync.5-4.prism",
            "",
            4244,
            5267,
        ),
        (
            59,
            "nand(20,1)",
            "dtmc/nand/nand.prism",
            "N=20;K=1",
            78332,
            121512,
        ),
        (
            120,
            "nand(20,2)",
            "dtmc/nand/nand.prism",
            "N=20;K=2",
            154942,
            239832,
        ),
        (
            185,
            "nand(20,3)",
            "dtmc/nand/nand.prism",
            "N=20;K=3",
            231552,
            358152,
        ),
        (
            240,
            "nand(20,4)",
            "dtmc/nand/nand.prism",
            "N=20;K=4",
            308162,
            476472,
        ),
        (
            877,
            "nand(40,1)",
            "dtmc/nand/nand.prism",
            "N=40;K=1",
            1004862,
            1581422,
        ),
        (
            1731,
            "nand(40,2)",
            "dtmc/nand/nand.prism",
            "N=40;K=2",
            2003082,
            3150462,
        ),
        (
            2482,
            "nand(40,3)",
            "dtmc/nand/nand.prism",
            "N=40;K=3",
            3001302,
            4719502,
        ),
        (
            3523,
            "nand(40,4)",
            "dtmc/nand/nand.prism",
            "N=40;K=4",
            3999522,
            6288542,
        ),
        (
            4161,
            "nand(60,1)",
            "dtmc/nand/nand.prism",
            "N=60;K=1",
            4717592,
            7463732,
        ),
        (
            9141,
            "nand(60,2)",
            "dtmc/nand/nand.prism",
            "N=60;K=2",
            9420422,
            14899892,
        ),
        (
            14899,
            "nand(60,3)",
            "dtmc/nand/nand.prism",
            "N=60;K=3",
            14123252,
            22336052,
        ),
        (
            28857,
            "nand(60,4)",
            "dtmc/nand/nand.prism",
            "N=60;K=4",
            18826082,
            29772212,
        ),
        (
            30382,
            "nand(60,4)",
            "dtmc/nand/nand.prism",
            "N=60;K=4",
            18826082,
            29772212,
        ),
        (
            12,
            "oscillators(6,6,0.1,1,0.1,1)",
            "dtmc/oscillators/oscillators.6-6-0.1-1.prism",
            "mu=0.1,lambda=1.0",
            463,
            1277,
        ),
        (
            159,
            "oscillators(6,8,0.1,1,0.1,1)",
            "dtmc/oscillators/oscillators.6-8-0.1-1.prism",
            "mu=0.1,lambda=1.0",
            1717,
            4726,
        ),
        (
            1412,
            "oscillators(6,10,0.1,1,0.1,1)",
            "dtmc/oscillators/oscillators.6-10-0.1-1.prism",
            "mu=0.1,lambda=1.0",
            5006,
            13598,
        ),
        (
            7877,
            "oscillators(7,10,0.1,1,0.1,1)",
            "dtmc/oscillators/oscillators.7-10-0.1-1.prism",
            "mu=0.1,lambda=1.0",
            11441,
            33459,
        ),
        (
            2462,
            "oscillators(8,8,0.1,1,0.1,1)",
            "dtmc/oscillators/oscillators.8-8-0.1-1.prism",
            "mu=0.1,lambda=1.0",
            6436,
            20525,
        ),
        (
            44482,
            "oscillators(8,10,0.1,1,0.1,1)",
            "dtmc/oscillators/oscillators.8-10-0.1-1.prism",
            "mu=0.1,lambda=1.0",
            24311,
            76623,
        ),
    ];

    for (reference_runtime, name, path, consts, states, _transitions) in models {
        if reference_runtime < 10_000 {
            let time = model_building::<MdpType>(path, consts, states);
            time.print(name, reference_runtime as f64 * 0.001);
        }
    }
}

struct BenchmarkResult {
    parsing: Duration,
    building: Duration,
    total: Duration,
    state_count: usize,
}

impl BenchmarkResult {
    fn print_header() {
        println!(
            "{:>30}    {:>10}   {:>10}   {:>10} {:>10} {:>12} {:>10}",
            "name", "parsing", "building", "total", "states", "per second", "relative"
        );
    }
    fn print(&self, name: &'static str, reference_time: f64) {
        println!(
            "{:>30}: {:>10}ms {:>10}ms {:>10}ms {:>10} {:>11.1}k {:>9.1}%",
            name,
            self.parsing.as_millis().to_string(),
            self.building.as_millis().to_string(),
            self.total.as_millis().to_string(),
            self.state_count.to_string(),
            self.state_count as f64 / self.building.as_secs_f64() * 0.001,
            (100.0 * (self.building.as_secs_f64() / reference_time)) - 100.0
        )
    }
}

fn model_building<M: ModelTypes>(
    path: &'static str,
    constants: &'static str,
    state_count: usize,
) -> BenchmarkResult {
    let start = std::time::Instant::now();

    let source = std::fs::read_to_string(format!("tiny-pmc-benchmarking/files/{}", path)).unwrap();
    let constants = tiny_pmc::parsing::parse_const_assignments(constants).unwrap();

    let parsed_model_and_objectives =
        tiny_pmc::parsing::parse_prism_and_print_errors::<&str>(Some(path), &source, &[]);
    let (mut prism_model, properties) = match parsed_model_and_objectives {
        None => panic!("Error parsing model"),
        Some((prism_model, properties, _)) => (prism_model, properties),
    };
    let parsing = start.elapsed();

    let mut atomic_propositions = Vec::new();
    let properties = tiny_pmc::building::prism_objectives_to_atomic_propositions(
        &mut atomic_propositions,
        properties,
    );
    let builder_output: ModelBuildingOutput<M> = prism_model_builder::build_model(
        &mut prism_model,
        &atomic_propositions[..],
        properties.into_iter(),
        &constants,
    )
    .unwrap();

    let total = start.elapsed();
    let building = total - parsing;
    let actual_state_count = builder_output.model.states.len();
    if actual_state_count != state_count {
        println!(
            "State counts don't match. Got {} states, while the reference implementation got {} states",
            actual_state_count, state_count
        );
    }
    BenchmarkResult {
        parsing,
        total,
        building,
        state_count: actual_state_count,
    }
}
