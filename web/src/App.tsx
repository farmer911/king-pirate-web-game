import React, { useEffect, useState } from "react";

export interface CharacterSummary {
  uid: number;
  uname: string;
  utid: number;
  level: number;
  gold_num: number;
  belly_num: number;
  vip: number;
}

export interface SessionInfo {
  loggedIn: boolean;
  id: number;
  accountName: string;
  dj: number;
  characters: CharacterSummary[];
}

type ModalType =
  | null
  | "login"
  | "reg"
  | "findpass"
  | "editpass"
  | "game_select"
  | "dh_exchange"
  | "level_tool"
  | "fix_tool"
  | "tg_promo"
  | "service_info";

function encodeForm(data: Record<string, string>): string {
  return Object.entries(data)
    .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
    .join("&");
}

export function App() {
  const [session, setSession] = useState<SessionInfo>({
    loggedIn: false,
    id: 0,
    accountName: "",
    dj: 0,
    characters: [],
  });
  const [loadingSession, setLoadingSession] = useState<boolean>(true);
  const [activeTab, setActiveTab] = useState<"game" | "yl" | "tools">("game");
  const [modal, setModal] = useState<ModalType>(null);
  const [playingServerFq, setPlayingServerFq] = useState<string | null>(null);
  const [spaPortalDrawerOpen, setSpaPortalDrawerOpen] = useState<boolean>(false);
  const [toast, setToast] = useState<{ text: string; ok: boolean } | null>(null);

  // Auth form states (matching app/sys/{login,reg,editpass,findpass}.php)
  const [uname, setUname] = useState("");
  const [upass, setUpass] = useState("");
  const [scode, setScode] = useState("");

  // Gold exchange states (matching app/dh/index.php)
  const [selectedFq, setSelectedFq] = useState<string>("");
  const [selectedRid, setSelectedRid] = useState<string>("");
  const [dhAmount, setDhAmount] = useState<string>("100");
  const [dhRoleHtmlLoaded, setDhRoleHtmlLoaded] = useState<boolean>(false);

  // Level tool states (matching legacy-ref/www/level.php)
  const [levelUname, setLevelUname] = useState<string>("");
  const [targetLevel, setTargetLevel] = useState<string>("200");
  const [logoutConfirmLevel, setLogoutConfirmLevel] = useState<boolean>(false);

  // Fix tool states (matching legacy-ref/www/fix.php)
  const [heroUname, setHeroUname] = useState<string>("");
  const [logoutConfirmHero, setLogoutConfirmHero] = useState<boolean>(false);
  const [bloodUname, setBloodUname] = useState<string>("");
  const [logoutConfirmBlood, setLogoutConfirmBlood] = useState<boolean>(false);

  const showToast = (text: string, ok = true) => {
    setToast({ text, ok });
    setTimeout(() => {
      setToast((prev) => (prev?.text === text ? null : prev));
    }, 3500);
  };

  const fetchSession = async () => {
    try {
      const res = await fetch("/api/session", {
        headers: { "Cache-Control": "no-cache" },
        credentials: "same-origin",
      });
      if (res.ok) {
        const data: SessionInfo = await res.json();
        setSession(data);
        if (data.characters && data.characters.length > 0) {
          const firstChar = data.characters[0].uname;
          setLevelUname((prev) => prev || firstChar);
          setHeroUname((prev) => prev || firstChar);
          setBloodUname((prev) => prev || firstChar);
          setSelectedRid(String(data.characters[0].uid));
        }
        return data;
      }
    } catch {
      // ignore transient error
    } finally {
      setLoadingSession(false);
    }
    return null;
  };

  useEffect(() => {
    document.body.className = "body_2";
    fetchSession().then((data) => {
      const path = window.location.pathname;
      const params = new URLSearchParams(window.location.search);
      if (path === "/app/Game/game.php") {
        setPlayingServerFq(params.get("FQ") || "1");
      } else if (path === "/app/Game/index.php") {
        setModal("game_select");
      } else if (path === "/app/dh/index.php") {
        setModal("dh_exchange");
      } else if (path === "/app/sys/login.php") {
        setModal("login");
      } else if (path === "/app/sys/reg.php") {
        setModal("reg");
      } else if (path === "/app/sys/editpass.php") {
        setModal("editpass");
      } else if (path === "/app/sys/findpass.php") {
        setModal("findpass");
      } else if (path === "/level.php") {
        setModal("level_tool");
      } else if (path === "/fix.php") {
        setModal("fix_tool");
      } else if (path === "/member.php" && data && !data.loggedIn) {
        window.history.replaceState({}, "", "/index.php");
      } else if ((path === "/" || path === "/index.php") && data && data.loggedIn) {
        window.history.replaceState({}, "", "/member.php");
      }
    });
  }, []);

  // Handlers for core/Cx_Ajax.php?type={login,reg,editpass,findpass,SignOut}
  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    if (uname.trim().length < 4 || upass.trim().length < 4) {
      showToast("Tài khoản và mật khẩu phải có ít nhất 4 ký tự!", false);
      return;
    }
    const res = await fetch("/core/Cx_Ajax.php?Action=Login", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: encodeForm({ Uname: uname.trim(), Upass: upass.trim() }),
      credentials: "same-origin",
    });
    const data = await res.json();
    if (data.status === "y") {
      showToast(data.info, true);
      setModal(null);
      await fetchSession();
      window.history.pushState({}, "", "/member.php");
    } else {
      showToast(data.info || "Đăng nhập không thành công!", false);
    }
  };

  const handleRegister = async (e: React.FormEvent) => {
    e.preventDefault();
    if (uname.trim().length < 4 || upass.trim().length < 4 || scode.trim().length < 4) {
      showToast("Tài khoản, mật khẩu và mã bảo mật phải từ 4 ký tự trở lên!", false);
      return;
    }
    const res = await fetch("/core/Cx_Ajax.php?Action=Register", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: encodeForm({
        Uname: uname.trim(),
        Upass: upass.trim(),
        Scode: scode.trim(),
      }),
      credentials: "same-origin",
    });
    const data = await res.json();
    if (data.status === "y") {
      showToast(data.info, true);
      setModal(null);
      await fetchSession();
      window.history.pushState({}, "", "/member.php");
    } else {
      showToast(data.info || "Đăng ký không thành công!", false);
    }
  };

  const handleFindPass = async (e: React.FormEvent) => {
    e.preventDefault();
    const res = await fetch("/core/Cx_Ajax.php?Action=FindPass", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: encodeForm({
        Uname: uname.trim(),
        Scode: scode.trim(),
        Upass: upass.trim(),
      }),
      credentials: "same-origin",
    });
    const data = await res.json();
    if (data.status === "y") {
      showToast(data.info, true);
      setModal("login");
    } else {
      showToast(data.info, false);
    }
  };

  const handleEditPass = async (e: React.FormEvent) => {
    e.preventDefault();
    const res = await fetch("/core/Cx_Ajax.php?Action=EditPass", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: encodeForm({
        Uname: session.accountName,
        Upass: upass.trim(),
        Scode: scode.trim(),
      }),
      credentials: "same-origin",
    });
    const data = await res.json();
    if (data.status === "y") {
      showToast(data.info, true);
      setModal(null);
      await fetchSession();
      window.history.pushState({}, "", "/index.php");
    } else {
      showToast(data.info, false);
    }
  };

  const handleSignOut = async () => {
    const res = await fetch("/core/Cx_Ajax.php?Action=SignOut", {
      method: "POST",
      credentials: "same-origin",
    });
    const data = await res.json();
    showToast(data.info || "Đăng xuất thành công!", true);
    setPlayingServerFq(null);
    setModal(null);
    await fetchSession();
    window.history.pushState({}, "", "/index.php");
  };

  // Handler for selecting server in Gold Exchange (app/dh/ajax.php?type=Get_GameUser)
  const handleSelectExchangeServer = async (fq: string) => {
    setSelectedFq(fq);
    const res = await fetch("/app/dh/ajax.php?Action=Get_GameUser", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: encodeForm({ FQ: fq }),
      credentials: "same-origin",
    });
    const data = await res.json();
    if (data.status === "y") {
      setDhRoleHtmlLoaded(true);
      await fetchSession();
    } else {
      showToast(data.info || data.html || "Không tìm thấy nhân vật!", false);
    }
  };

  // Handler for Gold Exchange submission (app/dh/ajax.php?Action=DHYB)
  const handleExchangeGold = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedFq) {
      showToast("Vui lòng chọn máy chủ!", false);
      return;
    }
    const res = await fetch("/app/dh/ajax.php?Action=DHYB", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: encodeForm({
        FQ: selectedFq,
        Rid: selectedRid || String(session.id),
        U_DH_Num: dhAmount.trim(),
      }),
      credentials: "same-origin",
    });
    const data = await res.json();
    if (data.status === "y") {
      showToast(data.info, true);
      await fetchSession();
    } else {
      showToast(data.info || "Đổi vàng thất bại!", false);
    }
  };

  // Handler for Level update (legacy-ref/www/level.php)
  const handleUpdateLevel = async (e: React.FormEvent) => {
    e.preventDefault();
    const body: Record<string, string> = {
      uname: levelUname.trim(),
      level: targetLevel.trim(),
    };
    if (logoutConfirmLevel) {
      body.logout_confirm = "on";
    }
    const res = await fetch("/api/level", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: encodeForm(body),
      credentials: "same-origin",
    });
    const data = await res.json();
    showToast(data.info, data.status === "y");
    if (data.status === "y") {
      await fetchSession();
    }
  };

  // Handler for Account/Copy Fix (legacy-ref/www/fix.php)
  const handleFixHero = async (e: React.FormEvent) => {
    e.preventDefault();
    const body: Record<string, string> = {
      form_type: "hero",
      uname_hero: heroUname.trim(),
    };
    if (logoutConfirmHero) {
      body.logout_confirm_hero = "on";
    }
    const res = await fetch("/api/fix", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: encodeForm(body),
      credentials: "same-origin",
    });
    const data = await res.json();
    showToast(data.info, data.status === "y");
    if (data.status === "y") {
      await fetchSession();
    }
  };

  const handleFixBlood = async (e: React.FormEvent) => {
    e.preventDefault();
    const body: Record<string, string> = {
      form_type: "blood",
      uname_blood: bloodUname.trim(),
    };
    if (logoutConfirmBlood) {
      body.logout_confirm_blood = "on";
    }
    const res = await fetch("/api/fix", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: encodeForm(body),
      credentials: "same-origin",
    });
    const data = await res.json();
    showToast(data.info, data.status === "y");
  };

  const sid = session.id || 202621270;

  return (
    <div className="main_bj">
      {toast && (
        <div
          className="portal-toast"
          style={{ background: toast.ok ? "#16a34a" : "#dc2626", zIndex: 20000 }}
        >
          {toast.text}
        </div>
      )}

      {/* Option B: Persistent Ruffle Game Canvas + Floating SPA HUD Overlay (/app/Game/game.php?FQ=1) */}
      {playingServerFq && (
        <div
          id="spa-game-shell"
          style={{
            position: "fixed",
            inset: 0,
            width: "100vw",
            height: "100vh",
            background: "#000",
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
            zIndex: 5000,
          }}
        >
          <div
            id="spa-game-hud-bar"
            style={{
              height: "40px",
              background: "linear-gradient(90deg, #003b5c 0%, #0088cc 100%)",
              color: "#fff",
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: "0 16px",
              fontSize: "13px",
              borderBottom: "1px solid #38bdf8",
              flexShrink: 0,
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
              <strong>[S{playingServerFq}.LouLx-Game] VUA HẢI TẶC 2023 (SPA)</strong>
              {session.accountName && (
                <span>
                  Tài khoản: <strong style={{ color: "#fde047" }}>{session.accountName}</strong> (SID:{" "}
                  {sid} • Điểm DJ: <strong style={{ color: "#fde047" }}>{session.dj.toLocaleString()}</strong>)
                </span>
              )}
              {session.characters.length > 0 && (
                <span style={{ background: "rgba(0,0,0,0.35)", padding: "2px 8px", borderRadius: "4px" }}>
                  Nhân vật: <strong>{session.characters[0].uname}</strong> (Lv.{session.characters[0].level} • VIP{" "}
                  {session.characters[0].vip} • Vàng: {session.characters[0].gold_num.toLocaleString()})
                </span>
              )}
            </div>
            <div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
              <button
                id="btn-hud-member-drawer"
                className="button button-little bg-yellow"
                onClick={() => {
                  setSpaPortalDrawerOpen((prev) => !prev);
                  fetchSession();
                }}
              >
                {spaPortalDrawerOpen ? "Đóng Bảng Thành Viên" : "Trung tâm thành viên (Overlay)"}
              </button>
              <button
                id="btn-hud-dh"
                className="button button-little bg-blue"
                onClick={() => {
                  setModal("dh_exchange");
                  handleSelectExchangeServer(playingServerFq);
                }}
              >
                Đổi Vàng
              </button>
              <button
                id="btn-hud-level"
                className="button button-little bg-green"
                onClick={() => {
                  fetchSession();
                  setModal("level_tool");
                }}
              >
                Cập nhật Level
              </button>
              <button
                id="btn-hud-fix"
                className="button button-little bg-sub"
                onClick={() => {
                  fetchSession();
                  setModal("fix_tool");
                }}
              >
                Sửa lỗi Nhân vật &amp; PB
              </button>
              <button
                id="btn-hud-exit-game"
                className="button button-little bg-red"
                onClick={() => {
                  setPlayingServerFq(null);
                  setSpaPortalDrawerOpen(false);
                  window.history.pushState({}, "", "/member.php");
                  fetchSession();
                }}
              >
                Thoát Màn Hình Game
              </button>
            </div>
          </div>

          <iframe
            id="Main_Game"
            name="mainFrame"
            title="VUA HẢI TẶC 2023 LouLx Game"
            src={`/T9NuoYa/T9NuoYa.html?sid=${sid}`}
            style={{ flex: 1, width: "100%", border: "none" }}
            allowFullScreen
          />
        </div>
      )}

      {/* Top Header: Cx_Nav -> DH_Panel + User_Panel (Exact index.php & member.php structure) */}
      <div
        id="spa-portal-container"
        style={
          playingServerFq
            ? spaPortalDrawerOpen
              ? {
                  position: "fixed",
                  top: "48px",
                  left: "50%",
                  transform: "translateX(-50%)",
                  zIndex: 9000,
                  maxHeight: "calc(100vh - 64px)",
                  overflowY: "auto",
                  background: "rgba(15, 23, 42, 0.95)",
                  padding: "12px",
                  borderRadius: "8px",
                  boxShadow: "0 20px 50px rgba(0,0,0,0.75)",
                }
              : { display: "none" }
            : undefined
        }
      >
      <div className="Cx_Nav">
        <div className="DH_Panel float-left">
          <span className="DH_Panel_Badge">PHIÊN BẢN OFFLINE 2023</span>
          <h1 className="DH_Panel_Title">《VUA HẢI TẶC》 — LOULX GAME</h1>
          <p className="DH_Panel_Sub">
            Tặng 1.000.000.000 Điểm trao đổi • Tạo nhân vật nhận ngay VIP 9, 1.000.000 Vàng, 10.000.000 Beri
          </p>
        </div>

        <div className="User_Panel bg-main float-right">
          {!session.loggedIn ? (
            /* Logged-out User_Panel (legacy-ref/www/index.php:37-52) */
            <div className="user-top-row">
              <img
                src="/WebRes/img/logo1.png"
                className="margin-right radius-circle border"
                height={80}
                width={80}
                alt="Logo"
              />
              <div className="float-left text-left">
                <h4 className="text-white" style={{ margin: "0 0 8px 0" }}>
                  Đăng nhập tài khoản để vào trò chơi
                </h4>
                <p style={{ margin: "6px 0" }}>
                  <button
                    type="button"
                    id="btn-open-login"
                    onClick={() => setModal("login")}
                    className="button button-little bg-yellow"
                  >
                    Đăng Nhập
                  </button>{" "}
                  <button
                    type="button"
                    id="btn-open-reg"
                    onClick={() => setModal("reg")}
                    className="button button-little bg-red"
                  >
                    Đăng ký
                  </button>
                </p>
                <p style={{ margin: "6px 0 0 0", fontSize: "13px" }}>
                  <span>《VUA HẢI TẶC》 Phiên bản Offline</span>
                </p>
              </div>
            </div>
          ) : (
            /* Logged-in User_Panel (legacy-ref/www/member.php:48-67) */
            <div>
              <div className="user-top-row">
                <img
                  src="/WebRes/img/logo1.png"
                  className="margin-right radius-circle border"
                  height={76}
                  width={76}
                  alt="Avatar"
                />
                <div className="float-left text-left">
                  <h4 className="text-white" style={{ margin: "0 0 8px 0" }}>
                    Xin chào {session.accountName} !
                  </h4>
                  <div style={{ display: "flex", gap: "6px", flexWrap: "wrap" }}>
                    <button
                      type="button"
                      onClick={() => setModal("tg_promo")}
                      className="button button-little bg-blue"
                    >
                      Quảng Cáo
                    </button>
                    <button
                      type="button"
                      onClick={() => setModal("editpass")}
                      className="button button-little bg-yellow"
                    >
                      Đổi mật khẩu
                    </button>
                    <button
                      type="button"
                      id="btn-signout"
                      onClick={handleSignOut}
                      className="button button-little bg-red"
                    >
                      Thoát
                    </button>
                  </div>
                </div>
              </div>

              <div className="user-stats-row">
                <div className="W33 text-center float-left">
                  <strong className="text-big text-yellow">T.Viên</strong>
                  <br />
                  Cấp bậc
                </div>
                <div className="W33 text-center float-left">
                  <strong className="text-big text-yellow" id="member-dj-balance">
                    {session.dj.toLocaleString()}
                  </strong>
                  <br />
                  Số dư
                </div>
                <div className="W33 text-center float-left">
                  <strong className="text-big text-yellow">Chưa K.hoạt</strong>
                  <br />
                  Lưu điểm
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Main Content Container (legacy-ref/www/index.php:53-80 & member.php:68-127) */}
      <div id="Main_Content" className="bg border border-main">
        {!session.loggedIn ? (
          /* Home Page Content (Exact Verbatim from legacy-ref/www/index.php:54-79) */
          <div className="margin">
            <p
              style={{
                textAlign: "center",
                padding: "6px",
                marginBottom: "5px",
                background: "#169ADA",
                color: "#ffffff",
                fontSize: "18px",
                fontWeight: "bold",
              }}
            >
              VUA HẢI TẶC 2023
            </p>
            <blockquote className="border-main" style={{ padding: "12px 16px", margin: "10px 0" }}>
              <strong>
                Nội dung cập nhật:
                <br />
                1. Big Mom, Kaido hình người, Monkey D Dragon, Tezoro, CP0 Lucci, CP0 Kaku,
                <br />
                Hoàng hậu kimono, Luật quốc gia Wano, Chopper quốc gia Wano, Zoro quốc gia Wano, Kira quốc gia Wano, Kakutari, Quinn ba thảm họa, Hanayo Nami,
                <br />
                Germa Sanji, Gear 4 Snake Luffy, Gear 4 Ape Luffy, Three-Disaster Jack, Kozuki Oden, Golden Lion Shiki, hình ảnh mới của Marco,
                <br />
                Hình ảnh mới của Enel, hình ảnh mới của Nami, hình ảnh mới của Hawkeye, hình ảnh mới của các đồng đội khác, v.v... Tôi không đếm hết được, chỉ chơi một mình thôi.
                <br />
                2. Đã thêm thú cưỡi mới, Shimanu, Komachiyo, Zeus và Prometheus, cho phép bạn trải nghiệm nhiều niềm vui hơn khi cưỡi trên người khác! !
                <br />
                3. BOSS thế giới đã được sửa chữa, thời gian sự kiện ban đầu sẽ được đồng bộ hóa vào mỗi buổi chiều và tối nên tôi biết bạn không có thời gian để chơi.
                <br />
                4. Bản đồ đã được sửa chữa cho phiên bản Đêm Giáng Sinh, ngày nào cũng đầy đủ lương thực và pháo binh
                <br />
                <br />
                Lỗi đã biết:
                <br />
                1. Vui lòng không sửa đổi trước khi rời khỏi đấu trường, nếu không đấu trường có thể không xuất hiện
                <br />
                2. Sau khi mở Trái ác quỷ, bạn phải thoát khỏi trò chơi và vào lại
                <br />
                3. Lưu ý không thêm quá 2,1 tỷ đồng vàng Berry
                <br />
                4. Huấn luyện gặp vấn đề không hiển thị Đồng Đội đang huấn luyện
                <br />
                5. Một số Event không hoạt động
              </strong>
            </blockquote>
          </div>
        ) : (
          /* Logged-in Member Center Content (Verbatim from legacy-ref/www/member.php:56-226) */
          <div className="tab">
            <div className="tab-head">
              <strong>Trung tâm thành viên</strong>
              <ul className="tab-nav">
                <li className={activeTab === "game" ? "active" : ""}>
                  <button type="button" onClick={() => setActiveTab("game")}>
                    GAME
                  </button>
                </li>
                <li className={activeTab === "yl" ? "active" : ""}>
                  <button type="button" onClick={() => setActiveTab("yl")}>
                    Giải trí
                  </button>
                </li>
                <li className={activeTab === "tools" ? "active" : ""}>
                  <button type="button" onClick={() => setActiveTab("tools")}>
                    Cập nhật Level &amp; Sửa lỗi
                  </button>
                </li>
              </ul>
            </div>

            {activeTab === "game" && (
              <div className="tab-panel active" id="tab-start2">
                <div className="line-big">
                  {/* Card 1: Bắt đầu trò chơi -> /app/Game/index.php */}
                  <div
                    id="card-start-game"
                    className="member-card"
                    onClick={() => setModal("game_select")}
                  >
                    <img
                      src="/WebRes/img/gcds-dragon-512.png"
                      className="margin-right radius-circle border"
                      height={60}
                      width={60}
                      alt="Bắt đầu trò chơi"
                    />
                    <div className="text-left">
                      <strong className="text-big text-main">Bắt đầu trò chơi</strong>
                      <br />
                      <span className="text-small">Nhấp để chọn máy chủ</span>
                    </div>
                  </div>

                  {/* Card 2: Đổi và Nạp tiền -> /app/dh/index.php */}
                  <div
                    id="card-exchange-gold"
                    className="member-card"
                    onClick={() => {
                      setModal("dh_exchange");
                      handleSelectExchangeServer("1");
                    }}
                  >
                    <img
                      src="/WebRes/img/gift_256x256.png"
                      className="margin-right radius-circle border"
                      height={60}
                      width={60}
                      alt="Đổi và Nạp tiền"
                    />
                    <div className="text-left">
                      <strong className="text-big text-main">Đổi và Nạp tiền</strong>
                      <br />
                      <span className="text-small">Nhấp để xem thông tin chi tiết</span>
                    </div>
                  </div>

                  {/* Card 3: Webshop / Cập nhật Level (level.php) */}
                  <div
                    id="card-level-tool"
                    className="member-card"
                    onClick={() => setModal("level_tool")}
                  >
                    <img
                      src="/WebRes/img/shop256.png"
                      className="margin-right radius-circle border"
                      height={60}
                      width={60}
                      alt="Cập nhật Level"
                    />
                    <div className="text-left">
                      <strong className="text-big text-main">Cập nhật Level (1-200)</strong>
                      <br />
                      <span className="text-small">Tùy chỉnh Level nhân vật</span>
                    </div>
                  </div>

                  {/* Card 4: Sân chơi / Sửa giá trị tài khoản (fix.php) */}
                  <div
                    id="card-fix-tool"
                    className="member-card"
                    onClick={() => setModal("fix_tool")}
                  >
                    <img
                      src="/WebRes/img/disney_dax_pink_19.png"
                      className="margin-right radius-circle border"
                      height={60}
                      width={60}
                      alt="Sửa giá trị tài khoản"
                    />
                    <div className="text-left">
                      <strong className="text-big text-main">Sửa lỗi Nhân vật &amp; PB</strong>
                      <br />
                      <span className="text-small">Sửa lỗi vượt Lv.200 &amp; Sinh Mệnh</span>
                    </div>
                  </div>

                  {/* Card 5: Dịch vụ */}
                  <div
                    id="card-service"
                    className="member-card"
                    onClick={() => setModal("service_info")}
                  >
                    <img
                      src="/WebRes/img/receptionist.png"
                      className="margin-right radius-circle border"
                      height={60}
                      width={60}
                      alt="Dịch vụ"
                    />
                    <div className="text-left">
                      <strong className="text-big text-main">Dịch vụ</strong>
                      <br />
                      <span className="text-small">Nhấp để xem thông tin chi tiết</span>
                    </div>
                  </div>
                </div>

                {/* Display current characters of this account for clarity */}
                {session.characters.length > 0 && (
                  <div style={{ marginTop: "20px", padding: "12px 16px", background: "#f0f9ff", border: "1px solid #bae6fd", borderRadius: "6px" }}>
                    <strong className="text-main">Nhân vật đã tạo trong máy chủ S1.LouLx-Game:</strong>
                    <div style={{ display: "flex", gap: "12px", flexWrap: "wrap", marginTop: "8px" }}>
                      {session.characters.map((ch) => (
                        <div
                          key={ch.uid}
                          style={{
                            background: "#fff",
                            border: "1px solid #0088cc",
                            borderRadius: "4px",
                            padding: "6px 12px",
                            fontSize: "13px",
                          }}
                        >
                          <strong>{ch.uname}</strong> (UID: {ch.uid}) — Lv.{ch.level} • VIP {ch.vip} • Vàng: {ch.gold_num.toLocaleString()} • Beri: {ch.belly_num.toLocaleString()}
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}

            {activeTab === "yl" && (
              <div className="tab-panel active" id="tab-yl">
                <div className="line-big">
                  <div className="member-card" onClick={() => setModal("game_select")}>
                    <img
                      src="/WebRes/img/disney_dax_pink_19.png"
                      className="margin-right radius-circle border"
                      height={60}
                      width={60}
                      alt="Kênh giải trí"
                    />
                    <div className="text-left">
                      <strong className="text-big text-main">Kênh giải trí</strong>
                      <br />
                      <span className="text-small">Sân chơi Vua Hải Tặc</span>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeTab === "tools" && (
              <div className="tab-panel active" id="tab-tools">
                <div className="line-big">
                  <div className="member-card" onClick={() => setModal("level_tool")}>
                    <img
                      src="/WebRes/img/shop256.png"
                      className="margin-right radius-circle border"
                      height={60}
                      width={60}
                      alt="Level"
                    />
                    <div className="text-left">
                      <strong className="text-big text-main">Cập nhật Level nhân vật</strong>
                      <br />
                      <span className="text-small">Tùy chỉnh Level 1 - 200 (level.php)</span>
                    </div>
                  </div>
                  <div className="member-card" onClick={() => setModal("fix_tool")}>
                    <img
                      src="/WebRes/img/disney_dax_pink_19.png"
                      className="margin-right radius-circle border"
                      height={60}
                      width={60}
                      alt="Fix"
                    />
                    <div className="text-left">
                      <strong className="text-big text-main">Sửa giá trị tài khoản</strong>
                      <br />
                      <span className="text-small">Sửa lỗi Lv.200 &amp; Sinh mệnh PB (fix.php)</span>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Footer (legacy-ref/www/index.php:91 & member.php:232) */}
      <div className="bg-inverse" id="footer">
        <div className="Navbar text-center">
          <p style={{ margin: 0 }}>
            VUA HẢI TẶC &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;LOULX GAME
          </p>
        </div>
      </div>
      </div>

      {/* ==================== LAYER MODALS ==================== */}

      {/* 1. Login Modal (legacy-ref/www/app/sys/login.php) */}
      {modal === "login" && (
        <div className="layer-backdrop" onClick={() => setModal(null)}>
          <div
            className="layer-modal"
            style={{ width: "480px" }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="layer-header">
              <span>Đăng Nhập</span>
              <button className="layer-close" onClick={() => setModal(null)}>
                ×
              </button>
            </div>
            <div className="layer-body">
              <form id="Login" onSubmit={handleLogin}>
                <div className="form-group">
                  <div className="field">
                    <input
                      type="text"
                      className="input"
                      id="Uname"
                      name="Uname"
                      placeholder="Tài Khoản"
                      value={uname}
                      onChange={(e) => setUname(e.target.value)}
                      required
                    />
                  </div>
                </div>
                <div className="form-group">
                  <div className="field">
                    <input
                      type="password"
                      className="input"
                      id="Upass"
                      name="Upass"
                      placeholder="Mật Khẩu"
                      value={upass}
                      onChange={(e) => setUpass(e.target.value)}
                      required
                    />
                  </div>
                </div>
                <button
                  className="button bg-main margin-bottom button-block"
                  type="submit"
                  id="THTj"
                >
                  Đăng Nhập
                </button>
                <button
                  onClick={() => setModal("findpass")}
                  className="button bg-yellow margin-bottom button-block"
                  type="button"
                >
                  Lấy Lại Mật Khẩu
                </button>
                <button
                  onClick={() => setModal("reg")}
                  className="button bg-red button-block"
                  type="button"
                >
                  Đăng Ký Tài Khoản
                </button>
              </form>
            </div>
          </div>
        </div>
      )}

      {/* 2. Register Modal (legacy-ref/www/app/sys/reg.php) */}
      {modal === "reg" && (
        <div className="layer-backdrop" onClick={() => setModal(null)}>
          <div
            className="layer-modal"
            style={{ width: "480px" }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="layer-header">
              <span>Đăng ký</span>
              <button className="layer-close" onClick={() => setModal(null)}>
                ×
              </button>
            </div>
            <div className="layer-body">
              <form id="Register" onSubmit={handleRegister}>
                <div className="form-group">
                  <div className="field">
                    <input
                      type="text"
                      className="input"
                      id="Uname"
                      name="Uname"
                      placeholder="Tên tài khoản"
                      value={uname}
                      onChange={(e) => setUname(e.target.value)}
                      required
                    />
                  </div>
                </div>
                <div className="form-group">
                  <div className="field">
                    <input
                      type="password"
                      className="input"
                      id="Upass"
                      name="Upass"
                      placeholder="Mật khẩu đăng nhập"
                      value={upass}
                      onChange={(e) => setUpass(e.target.value)}
                      required
                    />
                  </div>
                </div>
                <div className="form-group">
                  <div className="field">
                    <input
                      type="text"
                      className="input"
                      id="Scode"
                      name="Scode"
                      placeholder="Mã bảo mật (dùng để lấy lại mật khẩu)"
                      value={scode}
                      onChange={(e) => setScode(e.target.value)}
                      required
                    />
                  </div>
                </div>
                <button
                  className="button bg-main margin-bottom button-block"
                  type="submit"
                  id="THTj"
                >
                  Đăng Ký Ngay
                </button>
                <button
                  onClick={() => setModal("login")}
                  className="button bg-yellow button-block"
                  type="button"
                >
                  Quay Lại Đăng Nhập
                </button>
              </form>
            </div>
          </div>
        </div>
      )}

      {/* 3. Find Password Modal (legacy-ref/www/app/sys/findpass.php) */}
      {modal === "findpass" && (
        <div className="layer-backdrop" onClick={() => setModal(null)}>
          <div
            className="layer-modal"
            style={{ width: "480px" }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="layer-header">
              <span>Lấy Lại Mật Khẩu</span>
              <button className="layer-close" onClick={() => setModal(null)}>
                ×
              </button>
            </div>
            <div className="layer-body">
              <form id="Findpass" onSubmit={handleFindPass}>
                <div className="form-group">
                  <input
                    type="text"
                    className="input"
                    name="Uname"
                    placeholder="Tài khoản"
                    value={uname}
                    onChange={(e) => setUname(e.target.value)}
                    required
                  />
                </div>
                <div className="form-group">
                  <input
                    type="text"
                    className="input"
                    name="Scode"
                    placeholder="Mã bảo mật"
                    value={scode}
                    onChange={(e) => setScode(e.target.value)}
                    required
                  />
                </div>
                <div className="form-group">
                  <input
                    type="password"
                    className="input"
                    name="Upass"
                    placeholder="Mật khẩu mới"
                    value={upass}
                    onChange={(e) => setUpass(e.target.value)}
                    required
                  />
                </div>
                <button className="button bg-main margin-bottom button-block" type="submit">
                  Lấy Lại Mật Khẩu
                </button>
                <button
                  onClick={() => setModal("login")}
                  className="button bg-yellow button-block"
                  type="button"
                >
                  Quay Lại Đăng Nhập
                </button>
              </form>
            </div>
          </div>
        </div>
      )}

      {/* 4. Edit Password Modal (legacy-ref/www/app/sys/editpass.php) */}
      {modal === "editpass" && (
        <div className="layer-backdrop" onClick={() => setModal(null)}>
          <div
            className="layer-modal"
            style={{ width: "480px" }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="layer-header">
              <span>Đổi mật khẩu</span>
              <button className="layer-close" onClick={() => setModal(null)}>
                ×
              </button>
            </div>
            <div className="layer-body">
              <form id="Editpass" onSubmit={handleEditPass}>
                <div className="form-group">
                  <input
                    type="text"
                    className="input"
                    name="Uname"
                    value={session.accountName}
                    readOnly
                  />
                </div>
                <div className="form-group">
                  <input
                    type="password"
                    className="input"
                    name="Upass"
                    placeholder="Mật khẩu mới"
                    value={upass}
                    onChange={(e) => setUpass(e.target.value)}
                    required
                  />
                </div>
                <div className="form-group">
                  <input
                    type="text"
                    className="input"
                    name="Scode"
                    placeholder="Mã bảo mật"
                    value={scode}
                    onChange={(e) => setScode(e.target.value)}
                    required
                  />
                </div>
                <button className="button bg-main margin-bottom button-block" type="submit">
                  Xác nhận sửa đổi
                </button>
              </form>
            </div>
          </div>
        </div>
      )}

      {/* 5. Server Selection Modal (legacy-ref/www/app/Game/index.php) */}
      {modal === "game_select" && (
        <div className="layer-backdrop" onClick={() => setModal(null)}>
          <div
            className="layer-modal"
            style={{ width: "800px", minHeight: "360px" }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="layer-header">
              <span>Bắt đầu trò chơi — Chọn máy chủ</span>
              <button className="layer-close" onClick={() => setModal(null)}>
                ×
              </button>
            </div>
            <div className="layer-body">
              <p className="text-main" style={{ fontWeight: 700, marginBottom: "14px" }}>
                Danh sách máy chủ đang hoạt động (Nhấp vào máy chủ để vào trò chơi):
              </p>
              <div className="line-big">
                <div className="x4 margin-bottom">
                  <button
                    type="button"
                    id="btn-enter-server-1"
                    onClick={() => {
                      setModal(null);
                      setPlayingServerFq("1");
                      window.history.pushState({}, "", "/app/Game/game.php?FQ=1");
                    }}
                    className="button button-block bg-main text-big"
                    style={{ padding: "16px 20px" }}
                  >
                    LouLx-Game <span className="badge bg-red">VUA HẢI TẶC</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 6. Gold Exchange Modal (legacy-ref/www/app/dh/index.php) */}
      {modal === "dh_exchange" && (
        <div className="layer-backdrop" onClick={() => setModal(null)}>
          <div
            className="layer-modal"
            style={{ width: "600px" }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="layer-header">
              <span>Đổi và Nạp tiền</span>
              <button className="layer-close" onClick={() => setModal(null)}>
                ×
              </button>
            </div>
            <div className="layer-body">
              <form className="form" id="DHYB" onSubmit={handleExchangeGold}>
                <p className="text-center" style={{ background: "#fffbeb", border: "1px solid #fde68a", padding: "10px", borderRadius: "4px" }}>
                  <strong style={{ color: "#d97706" }}>
                    Hướng dẫn đổi thưởng (tỷ lệ: 1 đồng vàng = 10.000 đồng vàng tích lũy)
                  </strong>
                  <br />
                  <strong className="text-main">
                    Để nhận Vàng trực tiếp sau khi đổi, hệ thống cộng thẳng vào nhân vật của bạn trong máy chủ.
                  </strong>
                </p>

                <div className="form-group">
                  <div className="label text-main">
                    <label>Chọn máy chủ</label>
                  </div>
                  <div className="field padding-left">
                    <div className="button-group border-main radio">
                      <label
                        className={`button margin-top ${selectedFq === "1" ? "active" : ""}`}
                        onClick={() => handleSelectExchangeServer("1")}
                      >
                        <input
                          name="FQ"
                          value="1"
                          type="radio"
                          checked={selectedFq === "1"}
                          onChange={() => handleSelectExchangeServer("1")}
                          style={{ marginRight: "6px" }}
                        />
                        S1.LouLx-Game
                      </label>
                    </div>
                  </div>
                </div>

                <div className="form-group">
                  <div className="label text-main">
                    <label>Chọn nhân vật trong máy chủ</label>
                  </div>
                  <div className="field padding-left">
                    <div className="button-group border-main radio" id="S_GameUser">
                      {dhRoleHtmlLoaded ? (
                        session.characters.length > 0 ? (
                          session.characters.map((ch) => (
                            <label
                              key={ch.uid}
                              className={`button margin-top ${
                                selectedRid === String(ch.uid) ? "active" : ""
                              }`}
                              onClick={() => setSelectedRid(String(ch.uid))}
                            >
                              <input
                                name="Rid"
                                value={String(ch.uid)}
                                type="radio"
                                checked={selectedRid === String(ch.uid)}
                                onChange={() => setSelectedRid(String(ch.uid))}
                                style={{ marginRight: "6px" }}
                              />
                              {ch.uname} (Lv.{ch.level} - Vàng: {ch.gold_num.toLocaleString()})
                            </label>
                          ))
                        ) : (
                          <label className="button margin-top active">
                            <input
                              name="Rid"
                              value={String(session.id)}
                              type="radio"
                              checked
                              readOnly
                              style={{ marginRight: "6px" }}
                            />
                            {session.accountName}
                          </label>
                        )
                      ) : (
                        <span className="text-small">Nhấp vào máy chủ S1.LouLx-Game ở trên để tải nhân vật</span>
                      )}
                    </div>
                  </div>
                </div>

                <div className="form-group">
                  <div className="label">
                    <label htmlFor="U_DH_Num">Số tiền trao đổi (Số điểm dư: {session.dj.toLocaleString()})</label>
                  </div>
                  <div className="field padding-left">
                    <input
                      type="number"
                      className="input text-center"
                      id="U_DH_Num"
                      name="U_DH_Num"
                      min={1}
                      max={10000000}
                      value={dhAmount}
                      onChange={(e) => setDhAmount(e.target.value)}
                      required
                    />
                  </div>
                </div>

                <button
                  className="button bg-main margin-bottom button-block"
                  disabled={!selectedFq}
                  type="submit"
                  id="THTj"
                >
                  Xác nhận trao đổi
                </button>
              </form>
            </div>
          </div>
        </div>
      )}

      {/* 7. Level Update Modal (legacy-ref/www/level.php) */}
      {modal === "level_tool" && (
        <div className="layer-backdrop" onClick={() => setModal(null)}>
          <div
            className="layer-modal"
            style={{ width: "460px" }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="layer-header">
              <span>Cập nhật Level nhân vật (1 - 200)</span>
              <button className="layer-close" onClick={() => setModal(null)}>
                ×
              </button>
            </div>
            <div className="layer-body">
              <form id="LevelForm" onSubmit={handleUpdateLevel}>
                <div className="form-group">
                  <div className="label">Tên nhân vật</div>
                  <input
                    type="text"
                    id="uname"
                    name="uname"
                    className="input"
                    placeholder="Nhập tên nhân vật trong game"
                    value={levelUname}
                    onChange={(e) => setLevelUname(e.target.value)}
                    required
                  />
                </div>
                <div className="form-group">
                  <div className="label">Level mong muốn (1 - 200)</div>
                  <input
                    type="number"
                    id="level"
                    name="level"
                    className="input"
                    min={1}
                    max={200}
                    placeholder="Nhập level (1-200)"
                    value={targetLevel}
                    onChange={(e) => setTargetLevel(e.target.value)}
                    required
                  />
                </div>
                <div className="form-group">
                  <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer" }}>
                    <input
                      type="checkbox"
                      name="logout_confirm"
                      checked={logoutConfirmLevel}
                      onChange={(e) => setLogoutConfirmLevel(e.target.checked)}
                    />
                    <span>Xác nhận đã thoát game</span>
                  </label>
                </div>
                <button type="submit" id="btn-submit-level" className="button bg-main button-block">
                  Cập nhật Level
                </button>
              </form>
            </div>
          </div>
        </div>
      )}

      {/* 8. Fix Account / Copy Modal (legacy-ref/www/fix.php) */}
      {modal === "fix_tool" && (
        <div className="layer-backdrop" onClick={() => setModal(null)}>
          <div
            className="layer-modal"
            style={{ width: "480px" }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="layer-header">
              <span>Sửa giá trị tài khoản (fix.php)</span>
              <button className="layer-close" onClick={() => setModal(null)}>
                ×
              </button>
            </div>
            <div className="layer-body">
              {/* Form 1: Sửa lỗi cấp độ vượt Lv.200 */}
              <div style={{ marginBottom: "20px", paddingBottom: "16px", borderBottom: "1px solid #e5e7eb" }}>
                <h4 style={{ margin: "0 0 10px 0", color: "#1f2937" }}>
                  Sửa lỗi cấp độ vượt Lv.200
                </h4>
                <form id="FixHeroForm" onSubmit={handleFixHero}>
                  <div className="form-group">
                    <div className="label">Tên nhân vật</div>
                    <input
                      type="text"
                      id="uname_hero"
                      name="uname_hero"
                      className="input"
                      placeholder="Nhập tên nhân vật"
                      value={heroUname}
                      onChange={(e) => setHeroUname(e.target.value)}
                      required
                    />
                  </div>
                  <div className="form-group">
                    <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer" }}>
                      <input
                        type="checkbox"
                        name="logout_confirm_hero"
                        checked={logoutConfirmHero}
                        onChange={(e) => setLogoutConfirmHero(e.target.checked)}
                      />
                      <span>Xác nhận đã thoát game</span>
                    </label>
                  </div>
                  <button type="submit" id="btn-submit-fix-hero" className="button bg-main button-block">
                    Cập nhật Cấp độ
                  </button>
                </form>
              </div>

              {/* Form 2: Sửa lỗi Sinh Mệnh vượt mức không đi PB được */}
              <div>
                <h4 style={{ margin: "0 0 10px 0", color: "#1f2937" }}>
                  Sửa lỗi Sinh Mệnh vượt mức không đi PB được
                </h4>
                <form id="FixBloodForm" onSubmit={handleFixBlood}>
                  <div className="form-group">
                    <div className="label">Tên nhân vật</div>
                    <input
                      type="text"
                      id="uname_blood"
                      name="uname_blood"
                      className="input"
                      placeholder="Nhập tên nhân vật"
                      value={bloodUname}
                      onChange={(e) => setBloodUname(e.target.value)}
                      required
                    />
                  </div>
                  <div className="form-group">
                    <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer" }}>
                      <input
                        type="checkbox"
                        name="logout_confirm_blood"
                        checked={logoutConfirmBlood}
                        onChange={(e) => setLogoutConfirmBlood(e.target.checked)}
                      />
                      <span>Xác nhận đã thoát game</span>
                    </label>
                  </div>
                  <button type="submit" id="btn-submit-fix-blood" className="button bg-main button-block">
                    Xóa giá trị Sinh Mệnh
                  </button>
                </form>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 9. Promo / Service Info Modals */}
      {modal === "tg_promo" && (
        <div className="layer-backdrop" onClick={() => setModal(null)}>
          <div className="layer-modal" style={{ width: "440px" }} onClick={(e) => e.stopPropagation()}>
            <div className="layer-header">
              <span>Liên kết Quảng Cáo</span>
              <button className="layer-close" onClick={() => setModal(null)}>
                ×
              </button>
            </div>
            <div className="layer-body">
              <p>Liên kết giới thiệu tài khoản của bạn:</p>
              <input
                type="text"
                className="input"
                readOnly
                value={`${window.location.origin}/?tid=${session.id}`}
              />
            </div>
          </div>
        </div>
      )}

      {modal === "service_info" && (
        <div className="layer-backdrop" onClick={() => setModal(null)}>
          <div className="layer-modal" style={{ width: "440px" }} onClick={(e) => e.stopPropagation()}>
            <div className="layer-header">
              <span>Dịch vụ Khách hàng</span>
              <button className="layer-close" onClick={() => setModal(null)}>
                ×
              </button>
            </div>
            <div className="layer-body">
              <p>
                <strong>VUA HẢI TẶC — LOULX GAME</strong>
              </p>
              <p>Hỗ trợ trực tuyến và cập nhật phiên bản tại: https://www.loulxgame.com</p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
