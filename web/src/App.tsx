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

  // Fullscreen Game View (/app/Game/game.php?FQ=1)
  if (playingServerFq) {
    const sid = session.id || 202621270;
    return (
      <div style={{ width: "100vw", height: "100vh", background: "#000", display: "flex", flexDirection: "column", overflow: "hidden" }}>
        <div
          style={{
            height: "36px",
            background: "linear-gradient(90deg, #004d73 0%, #0088cc 100%)",
            color: "#fff",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "0 16px",
            fontSize: "13px",
            borderBottom: "1px solid #38bdf8",
          }}
        >
          <div>
            <strong>[S{playingServerFq}.LouLx-Game] VUA HẢI TẶC</strong>
            {session.accountName ? ` — Tài khoản: ${session.accountName} (SID: ${sid})` : ""}
          </div>
          <div style={{ display: "flex", gap: "8px" }}>
            <button
              className="button button-little bg-yellow"
              onClick={() => {
                setPlayingServerFq(null);
                window.history.pushState({}, "", "/member.php");
                fetchSession();
              }}
            >
              Trung tâm thành viên
            </button>
            <button
              className="button button-little bg-blue"
              onClick={() => {
                setPlayingServerFq(null);
                setModal("dh_exchange");
                window.history.pushState({}, "", "/member.php");
              }}
            >
              Đổi Vàng
            </button>
            <button
              className="button button-little bg-green"
              onClick={() => {
                setPlayingServerFq(null);
                setModal("level_tool");
                window.history.pushState({}, "", "/member.php");
              }}
            >
              Cập nhật Level
            </button>
          </div>
        </div>
        <iframe
          id="Main_Game"
          title="VUA HẢI TẶC"
          src={`/T9NuoYa/T9NuoYa.html?sid=${sid}`}
          style={{ flex: 1, width: "100%", border: "none" }}
          allowFullScreen
        />
      </div>
    );
  }

  return (
    <div className="main_bj">
      {toast && (
        <div
          className="portal-toast"
          style={{ background: toast.ok ? "#16a34a" : "#dc2626" }}
        >
          {toast.text}
        </div>
      )}

      {/* Top Header: Cx_Nav -> DH_Panel + User_Panel (Exact index.php & member.php structure) */}
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
            /* Logged-out User_Panel (legacy-ref/www/index.php:17-37) */
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
                  Đăng nhập tài khoản để vào trò chơi!
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
            /* Logged-in User_Panel (legacy-ref/www/member.php:17-54) */
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

      {/* Main Content Container (legacy-ref/www/index.php & member.php) */}
      <div id="Main_Content" className="bg border border-main">
        {!session.loggedIn ? (
          /* Home Page Content (Verbatim from legacy-ref/www/index.php:40-87) */
          <div className="margin-big">
            <h4>
              <strong>VUA HẢI TẶC 2023</strong>
            </h4>
            <p className="bg-inverse">
              <strong>
                ** Lần đầu tạo tài khoản tặng 1000000000 điểm trao đổi ( dùng điểm này để đổi vàng trên web). Tạo nhân vật vào game nhận VIP 9, 1000000 Vàng, 10000000beri, 100000 Lực hành động.
              </strong>
            </p>
            <p>
              <strong>Nội dung cập nhật:</strong>
            </p>
            <p>1. Thêm danh hiệu mới: Ngũ hoàng, Vua Hải Tặc Tối Thượng, v.v.</p>
            <p>2. Đã thêm thời trang Wano Quốc và Cưỡi Linh thú mới.</p>
            <p>3. Giới hạn cấp độ là 200 và các bản sao mới của Punk Hassad và Đồi Râu Trắng được thêm vào.</p>
            <p>
              4. Các hoạt động trong trò chơi cực kỳ phong phú, chẳng hạn như Chiến tranh thế giới, Chiến đấu theo nhóm liên máy chủ và Chiến tranh bang hội.
            </p>
            <p>
              <strong>Lỗi đã biết:</strong>
            </p>
            <p>
              1. Nếu đăng nhập hiện thông báo bảo trì có thể do bạn chưa chạy (1) để bật server hoặc bật rồi nhưng vào quá nhanh, cần đợi 1-2p sau khi bật để các dịch vụ của game chạy hoàn tất.
            </p>
            <p>2. Hầu hết thời trang của các tướng là không có tác dụng ( ngoại trừ các tướng tự tạo)</p>
            <p>
              3. Do trò chơi là bản dựng lại và sử dụng kết hợp các phiên bản nên có thể có 1 số vật phẩm hoặc sự kiện bị lỗi, không hoạt động ( tỉ lệ rất nhỏ).
            </p>
            <p>
              4. Trò chơi có nhiều tướng Haki: Luffy Gear4, Kaido, BigMom, Râu Trắng, Shank, Mihawk,... có thể kiếm thông qua việc vượt các ải trong trò chơi ( không lấy sẵn được do giới hạn từ nhà phát hành).
            </p>
            <p>
              5. Do giới hạn của trò chơi, các tài nguyên như vàng hay beri không nên vượt quá 100 triệu để tránh bị âm, nếu âm cách duy nhất là tạo nhân vật mới.
            </p>
            <p>
              <strong>
                Mua Game Tại :{" "}
                <a href="https://www.loulxgame.com" target="_blank" rel="noreferrer">
                  LouLx Game
                </a>
              </strong>
            </p>
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
