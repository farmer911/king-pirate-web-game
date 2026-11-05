//! Zero-allocation RPC Domain Router (`src/domain/mod.rs`).
//!
//! Consolidates the `106` legacy PHP modules (`2,070` files in `legacy-ref/rpcfw/module/`)
//! into 11 cohesive, strongly-typed Rust Bounded Domains with zero dead code.

pub mod arena;
pub mod bag;
pub mod boss;
pub mod copy;
pub mod economy;
pub mod forge;
pub mod guild;
pub mod hero;
pub mod pet;
pub mod town;
pub mod user;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amf3::Amf3Value;
    use crate::battle::{FormationGrid, HeroUnit};
    use crate::btstore::StaticBtstore;
    use crate::crypt::{babel_encrypt_number, compute_login_play_hash};
    use crate::data::{DataStore, SqlDialect};
    use crate::gateway::ConnectionSession;
    use std::collections::BTreeMap;

    #[test]
    fn test_end_to_end_login_create_character_gameplay_and_write_behind() {
        let store = DataStore::new(SqlDialect::SqliteWal);
        let btstore = StaticBtstore::new_baseline();
        let mut session = ConnectionSession::default();

        // 1. Authenticate `user.login` with verified PlayHashKey & DES-CBC pid
        let enc_pid = babel_encrypt_number(202621270);
        let mut login_params = BTreeMap::new();
        login_params.insert("openDateTime".to_string(), "2012-05-12".to_string());
        login_params.insert("pid".to_string(), enc_pid);
        login_params.insert("ptype".to_string(), "0".to_string());
        login_params.insert("timestamp".to_string(), "1357000000".to_string());
        let hash = compute_login_play_hash(&login_params);
        login_params.insert("hash".to_string(), hash);

        let login_res = user::rpc_user_login(&mut session, &login_params, "20120512", 1357000050);
        assert_eq!(login_res, Amf3Value::String("ok".to_string()));
        assert_eq!(session.pid, Some(202621270));

        // 2. Create character `user.createUser` -> spawns uid 20001
        let (uid, create_res) =
            user::rpc_create_user(&session, &store, 1, "MonkeyDLuffy").expect("create_user failed");
        assert_eq!(uid, 20001);
        assert_eq!(create_res, Amf3Value::String("ok".to_string()));

        // 3. Verify `user.getUsers` returns the newly created character
        let users_val = user::rpc_get_users(&session, &store).expect("get_users failed");
        match users_val {
            Amf3Value::Array { dense, .. } => assert_eq!(dense.len(), 1),
            _ => panic!("expected dense array from getUsers"),
        }

        // 4. Enter game `user.userLogin` -> spawns In-Memory `PlayerSessionActor`
        let (mut actor, ulogin_res) =
            user::rpc_user_character_login(&mut session, &store, uid).expect("userLogin failed");
        assert_eq!(ulogin_res, Amf3Value::String("ok".to_string()));
        assert_eq!(session.uid, Some(20001));

        // 5. Execute gameplay across all bounded domains in L1 RAM (`~50 ns`)
        let _ = economy::daily_sign_reward(&mut actor, 50, 2000);
        let _ = economy::kitchen_cook(&mut actor, 300);
        assert_eq!(user::rpc_buy_execution(&mut actor, 10, 20), Amf3Value::String("ok".to_string()));

        // Bag + Forge domain
        let mut pbag = bag::PlayerBag::new(24);
        assert!(pbag.open_grid_by_gold(&mut actor, 2, 10));
        let slot = pbag.add_item(&store, 10001, 1).expect("add_item failed");
        actor.level = 5;
        let item = pbag.grids.get_mut(&slot).expect("item missing");
        let forge_res = forge::reinforce_item(&mut actor, item, 200);
        assert_eq!(forge_res.get_field("ret").and_then(|v| v.as_str()), Some("ok"));
        assert_eq!(item.reinforce_level, 1);
        assert!(pbag.to_amf3().get_field("max_grid_num").is_some());

        // Copy + Arena + Boss domain
        let mut p_grid = FormationGrid::new(actor.uid, actor.cur_formation);
        p_grid.set_hero(0, HeroUnit::new_basic(101, 1, 0, 10, 1500, 400, 120));
        let copy_res = copy::attack_copy_enemy(&mut actor, p_grid.clone(), 1001, 1, &btstore)
            .expect("attack_copy failed");
        assert_eq!(
            copy_res.get_field("appraisal").and_then(|v| v.as_str()),
            Some("A")
        );

        let mut d_grid = FormationGrid::new(20002, 10002);
        d_grid.set_hero(0, HeroUnit::new_basic(201, 2, 0, 2, 500, 100, 40));
        let mut my_pos = 50u32;
        let mut opp_pos = 10u32;
        let arena_res = arena::challenge_arena_opponent(
            &mut actor,
            p_grid,
            d_grid,
            &mut my_pos,
            &mut opp_pos,
            1357001000,
            &btstore,
        )
        .expect("arena challenge failed");
        assert_eq!(
            arena_res.get_field("appraisal").and_then(|v| v.as_str()),
            Some("A")
        );
        assert_eq!(my_pos, 10);

        let mut boss = boss::WorldBossState::new(1, 50, 10000);
        let boss_res = boss.attack_boss(&mut actor, 2500);
        assert_eq!(boss_res.get_field("boss_hp").and_then(|v| v.as_i64()), Some(7500));

        // Hero + Guild + Town + Pet domain
        let mut hero_rec =
            hero::recruit_hero(&mut actor, &store, 101, 10).expect("recruit_hero failed");
        assert!(hero::add_goodwill_by_gold(&mut actor, &mut hero_rec, 5));
        assert!(hero::upgrade_haki_by_belly(&mut actor, &mut hero_rec, 100));
        assert!(hero_rec.to_battle_unit(0).max_hp > 800);
        assert!(hero_rec.to_amf3().get_field("hid").is_some());

        let mut guild_rec =
            guild::create_guild(&mut actor, &store, "StrawHat", 500).expect("create_guild failed");
        assert_eq!(
            guild::contribute_belly(&mut actor, &mut guild_rec, 200),
            Amf3Value::String("ok".to_string())
        );

        let move_res = town::move_in_town(&mut actor, 1, 25, 30);
        assert_eq!(move_res.get_field("x").and_then(|v| v.as_i64()), Some(25));
        assert_eq!(town::unpack_town_xy((25 << 16) | 30), (25, 30));

        let mut pet_rec = pet::PetRecord {
            pet_id: 1,
            pet_tmpl_id: 1,
            level: 1,
            exp: 0,
            evolved: false,
        };
        let pet_res = pet::train_pet(&mut actor, &mut pet_rec, 150, 100);
        assert_eq!(pet_res.get_field("level").and_then(|v| v.as_i64()), Some(2));

        // 6. Flush Write-Behind delta from `PlayerSessionActor` to `DataStore`
        let delta = actor.take_dirty_delta().expect("expected dirty delta");
        assert_eq!(store.apply_write_behind_delta(&delta), 1);
    }
}
