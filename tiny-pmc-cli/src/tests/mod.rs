use crate::input::prism::parse_prism;

#[test]
fn parse_tiny_test() {
    let model = parse_prism(None, include_str!("files/tiny_test.prism")).unwrap();
    println!("===========\nMODEL:\n===========");
    println!("{}", model);
    let built = prism_model_builder::build_model(&model);
}

#[test]
fn parse_bluetooth() {
    parse_prism(None, include_str!("files/bluetooth.v1.prism")).unwrap();
}

#[test]
fn parse_brp() {
    parse_prism(None, include_str!("files/brp.v1.prism")).unwrap();
}

#[test]
fn parse_cluster() {
    parse_prism(None, include_str!("files/cluster.v1.prism")).unwrap();
}

#[test]
fn parse_consensus_2() {
    parse_prism(None, include_str!("files/consensus.2.v1.prism")).unwrap();
}

#[test]
fn parse_crowds() {
    parse_prism(Some("crowds"), include_str!("files/crowds.v1.prism")).unwrap();
}

#[test]
fn parse_csma() {
    parse_prism(None, include_str!("files/csma.2-2.v1.prism")).unwrap();
}

#[test]
fn parse_eajs_2() {
    parse_prism(None, include_str!("files/eajs.2.v1.prism")).unwrap();
}

#[test]
fn parse_egl() {
    parse_prism(None, include_str!("files/egl.v1.prism")).unwrap();
}

#[test]
fn parse_embedded() {
    parse_prism(None, include_str!("files/embedded.v1.prism")).unwrap();
}

#[test]
fn parse_firewire_false() {
    parse_prism(None, include_str!("files/firewire.false.v2.prism")).unwrap();
}

#[test]
fn parse_firewire_true() {
    parse_prism(None, include_str!("files/firewire.true.v2.prism")).unwrap();
}

#[test]
fn parse_firewire_abst() {
    parse_prism(None, include_str!("files/firewire_abst.v1.prism")).unwrap();
}

#[test]
fn parse_firewire_dl() {
    parse_prism(None, include_str!("files/firewire_dl.v1.prism")).unwrap();
}

#[test]
fn parse_fms() {
    parse_prism(None, include_str!("files/fms.v1.prism")).unwrap();
}

#[test]
fn parse_haddad_monmege() {
    parse_prism(None, include_str!("files/haddad-monmege.v1.prism")).unwrap();
}

#[test]
fn parse_herman_3() {
    parse_prism(None, include_str!("files/herman.3.v1.prism")).unwrap();
}

#[test]
fn parse_herman_5() {
    parse_prism(None, include_str!("files/herman.5.v1.prism")).unwrap();
}

#[test]
fn parse_herman_7() {
    parse_prism(None, include_str!("files/herman.7.v1.prism")).unwrap();
}

#[test]
fn parse_herman_9() {
    parse_prism(None, include_str!("files/herman.9.v1.prism")).unwrap();
}

#[test]
fn parse_hill_toggle() {
    parse_prism(None, include_str!("files/hill-toggle.v1.prism")).unwrap();
}

#[test]
fn parse_ij_3() {
    parse_prism(None, include_str!("files/ij.3.v1.prism")).unwrap();
}

#[test]
fn parse_ij_10() {
    parse_prism(None, include_str!("files/ij.10.v1.prism")).unwrap();
}

#[test]
fn parse_kanban() {
    parse_prism(None, include_str!("files/kanban.v1.prism")).unwrap();
}

#[test]
fn parse_leader_sync() {
    parse_prism(None, include_str!("files/leader_sync.3-2.v1.prism")).unwrap();
}

#[test]
fn parse_majority() {
    parse_prism(None, include_str!("files/majority.v1.prism")).unwrap();
}

#[test]
fn parse_nand() {
    parse_prism(None, include_str!("files/nand.v1.prism")).unwrap();
}

#[test]
fn parse_oscillators() {
    parse_prism(None, include_str!("files/oscillators.3-6-0.1-1.v1.prism")).unwrap();
}

#[test]
fn parse_p53() {
    parse_prism(None, include_str!("files/p53.v1.prism")).unwrap();
}

#[test]
fn parse_pacman() {
    parse_prism(None, include_str!("files/pacman.v2.prism")).unwrap();
}

#[test]
fn parse_philosophers() {
    parse_prism(None, include_str!("files/philosophers-mdp.3.v1.prism")).unwrap();
}

#[test]
fn parse_pnueli_zuck() {
    parse_prism(None, include_str!("files/pnueli-zuck.3.v1.prism")).unwrap();
}

#[test]
fn parse_polling() {
    parse_prism(None, include_str!("files/polling.3.v1.prism")).unwrap();
}

#[test]
fn parse_rabin_3() {
    parse_prism(None, include_str!("files/rabin.3.v1.prism")).unwrap();
}

#[test]
fn parse_rabin_5() {
    parse_prism(None, include_str!("files/rabin.5.v1.prism")).unwrap();
}

#[test]
fn parse_resource_gathering() {
    parse_prism(None, include_str!("files/resource-gathering.v2.prism")).unwrap();
}

#[test]
fn parse_speed_ind() {
    parse_prism(None, include_str!("files/speed-ind.v1.prism")).unwrap();
}

#[test]
fn parse_tandem() {
    parse_prism(None, include_str!("files/tandem.v1.prism")).unwrap();
}

#[test]
fn parse_toggle_switch() {
    parse_prism(None, include_str!("files/toggle-switch.v1.prism")).unwrap();
}

#[test]
fn parse_wlan_0() {
    parse_prism(None, include_str!("files/wlan.0.v1.prism")).unwrap();
}

#[test]
fn parse_wlan_1() {
    parse_prism(None, include_str!("files/wlan.1.v1.prism")).unwrap();
}

#[test]
fn parse_wlan_dl_0() {
    parse_prism(None, include_str!("files/wlan_dl.0.v1.prism")).unwrap();
}

#[test]
fn parse_wlan_dl_1() {
    parse_prism(None, include_str!("files/wlan_dl.1.v1.prism")).unwrap();
}

#[test]
fn parse_zeroconf() {
    parse_prism(None, include_str!("files/zeroconf.v1.prism")).unwrap();
}

#[test]
fn parse_zeroconf_dl() {
    parse_prism(None, include_str!("files/zeroconf_dl.v1.prism")).unwrap();
}
