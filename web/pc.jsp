<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<%@include file="header1.inc"%>
<%@include file="common.inc"%>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <title>《天下AI网游》- 界面选择</title>
    <link href="includes/modern-gaming.css" rel="stylesheet" type="text/css"/>
    <style>
        .ui-selection {
            display: flex;
            flex-direction: column;
            gap: 20px;
            margin-top: 30px;
        }

        .ui-title {
            text-align: center;
            margin-bottom: 20px;
        }

        .ui-title h2 {
            color: var(--text-primary);
            margin-bottom: 8px;
        }

        .ui-title p {
            color: var(--text-secondary);
            font-size: 14px;
        }

        .ui-cards {
            display: flex;
            gap: 20px;
            flex-wrap: wrap;
        }

        .ui-card {
            flex: 1;
            min-width: 250px;
            padding: 25px;
            border-radius: 12px;
            cursor: pointer;
            transition: all 0.3s ease;
            border: 2px solid transparent;
        }

        .ui-card:hover {
            transform: translateY(-5px);
            box-shadow: 0 10px 30px rgba(0, 255, 255, 0.3);
        }

        .ui-card.new-ui {
            background: linear-gradient(135deg, rgba(0, 255, 255, 0.1) 0%, rgba(102, 126, 234, 0.1) 100%);
            border-color: rgba(0, 255, 255, 0.3);
        }

        .ui-card.new-ui:hover {
            border-color: var(--primary-cyan);
        }

        .ui-card.old-ui {
            background: linear-gradient(135deg, rgba(139, 119, 101, 0.1) 0%, rgba(107, 91, 71, 0.1) 100%);
            border-color: rgba(139, 119, 101, 0.3);
        }

        .ui-card.old-ui:hover {
            border-color: #8B7765;
        }

        .ui-card-icon {
            font-size: 48px;
            text-align: center;
            margin-bottom: 15px;
        }

        .ui-card-title {
            text-align: center;
            font-size: 18px;
            font-weight: bold;
            margin-bottom: 10px;
            color: var(--text-primary);
        }

        .ui-card-desc {
            text-align: center;
            font-size: 13px;
            color: var(--text-secondary);
            line-height: 1.6;
        }

        .ui-card-features {
            margin-top: 15px;
            font-size: 12px;
            color: var(--text-secondary);
        }

        .ui-card-features li {
            margin-bottom: 5px;
            padding-left: 15px;
            position: relative;
        }

        .ui-card-features li:before {
            content: "✓";
            position: absolute;
            left: 0;
            color: var(--primary-cyan);
        }

        .ui-btn {
            margin-top: 15px;
            width: 100%;
        }

        .remember-choice {
            text-align: center;
            margin-top: 20px;
            padding-top: 15px;
            border-top: 1px solid rgba(0, 255, 255, 0.2);
        }

        .remember-choice label {
            color: var(--text-secondary);
            font-size: 13px;
            cursor: pointer;
            display: inline-flex;
            align-items: center;
            gap: 8px;
        }

        .remember-choice input[type="checkbox"] {
            width: 16px;
            height: 16px;
            cursor: pointer;
        }

        @media (max-width: 600px) {
            .ui-cards {
                flex-direction: column;
            }

            .ui-card {
                min-width: 100%;
            }
        }

        /* 隐藏旧登录表单 */
        .old-login-form {
            display: none;
        }

        .login-form {
            display: flex;
            flex-direction: column;
            gap: 20px;
        }

        .login-buttons {
            display: flex;
            gap: 12px;
            margin-top: 20px;
        }

        .btn-login {
            flex: 1;
            position: relative;
            z-index: 1;
        }

        .register-link {
            text-align: center;
            margin-top: 20px;
            padding-top: 20px;
            border-top: 1px solid rgba(0, 255, 255, 0.2);
        }

        .register-link p {
            color: var(--text-secondary) !important;
        }

        .register-link a {
            color: var(--primary-cyan);
            text-shadow: 0 0 10px rgba(0, 255, 255, 0.3);
        }

        .register-link a:hover {
            color: #00ffff;
            text-shadow: 0 0 20px rgba(0, 255, 255, 0.8);
        }

        .footer {
            margin-top: 40px;
            padding: 20px;
            text-align: center;
            color: var(--text-secondary);
            border-top: 1px solid rgba(0, 255, 255, 0.2);
            font-size: 12px;
        }

        @media (max-width: 480px) {
            .login-buttons {
                flex-direction: column;
            }

            .btn {
                width: 100%;
            }

            .register-link {
                padding-top: 15px;
            }
        }
    </style>
    <%=favicon%>
</head>
<body class="classical-pattern">
    <div class="header">
        <div class="container-narrow">
            <img src="logo-1.gif" class="logo">
            <div class="header-title">《天下AI网游》</div>
            <div class="header-subtitle">原版天下 • 醇香文字 • 邀你同行</div>
        </div>
    </div>

    <div class="container-narrow">
        <!-- 界面选择 -->
        <div class="login-card" id="uiSelection">
            <div class="ui-title">
                <h2>选择游戏界面</h2>
                <p>请选择您喜欢的游戏界面风格</p>
            </div>

            <div class="ui-cards">
                <div class="ui-card new-ui" onclick="selectUI('new')">
                    <div class="ui-card-icon">🚀</div>
                    <div class="ui-card-title">新版界面</div>
                    <div class="ui-card-desc">现代化 Vue 界面，响应式设计</div>
                    <ul class="ui-card-features">
                        <li>流畅的动画效果</li>
                        <li>移动端友好</li>
                        <li>密码哈希加密</li>
                        <li>实时速率保护</li>
                    </ul>
                </div>

                <div class="ui-card old-ui" onclick="selectUI('old')">
                    <div class="ui-card-icon">📜</div>
                    <div class="ui-card-title">经典界面</div>
                    <div class="ui-card-desc">传统 JSP 界面，怀旧经典</div>
                    <ul class="ui-card-features">
                        <li>稳定可靠</li>
                        <li>低配设备友好</li>
                        <li>保留原有体验</li>
                        <li>轻量级加载</li>
                    </ul>
                </div>
            </div>

            <div class="remember-choice">
                <label>
                    <input type="checkbox" id="rememberChoice" checked>
                    记住我的选择
                </label>
            </div>
        </div>

        <!-- 旧版登录表单 (隐藏，选择旧界面时显示) -->
        <div class="login-card old-login-form" id="oldLoginForm">
            <h2 style="text-align: center; margin-bottom: 30px;">账号登录</h2>
            <div style="text-align: center; margin-bottom: 20px;">
                <a href="javascript:void(0);" onclick="showUISelection()" style="color: var(--primary-cyan);">← 返回界面选择</a>
            </div>

            <%
                String m_key = request.getParameter("m_key");
                String mid = request.getParameter("mid");
                if(mid==null){
                    long ot = System.currentTimeMillis();
                    mid=String.valueOf(ot);
                }
                if(m_key==null){
                    long ot = System.currentTimeMillis();
                    m_key=String.valueOf(ot);
                }

                String z = (String)request.getParameter("z");
                if(z==null)
                    z=(String)request.getSession().getAttribute("z");
                else
                    request.getSession().setAttribute("z",z);

                String error_str = request.getParameter("err");
                String p_user = request.getParameter("_user");
                String p_pswd = request.getParameter("_pswd");
                if(p_user == null)
                    p_user = "";
                if(p_pswd == null)
                    p_pswd = "";

                if("1".equals(error_str))
                    out.print("<div class='error-message'>❌ 用户名和密码不能为空，请修改后重试</div>");
                else if("2".equals(error_str))
                    out.print("<div class='error-message'>❌ 用户名和密码不能少于2个字符，请修改后重试</div>");
                else if("3".equals(error_str))
                    out.print("<div class='error-message'>❌ 用户名和密码只能是大小写字母或数字，请修改后重试</div>");
                else if("5".equals(error_str))
                    out.print("<div class='error-message'>❌ 游戏账号和密码必须是2~12位的英文或者数字，或者两者的组合</div>");
                else if("4".equals(error_str))
                    out.print("<div class='error-message'>❌ 您输入的用户名和密码验证失败或有人正在使用该帐号</div>");
                else if("6".equals(error_str))
                    out.print("<div class='error-message'>❌ 您输入的用户名和密码验证失败，是否需要找回密码？</div>");
                else if("7".equals(error_str))
                    out.print("<div class='error-message'>❌ 系统犯晕了，请通知管理员</div>");
            %>

            <form action="./entrycheck.jsp?regnewFlag=0" method="post" class="login-form">
                <div class="form-group">
                    <label class="form-label">账号所属区</label>
                    <select name="_game_pre" id="game_pre" class="form-control">
                        <%
                            String gameArea = System.getenv("GAME_AREA");
                            if (gameArea == null || gameArea.isEmpty()) {
                                gameArea = "01";
                            }
                            if (gameArea.startsWith("tx")) {
                                gameArea = gameArea.substring(2);
                            }
                            int startArea, endArea;
                            if (gameArea.contains("-")) {
                                String[] parts = gameArea.split("-");
                                startArea = Integer.parseInt(parts[0].replaceAll("^0+", "").isEmpty() ? "0" : parts[0].replaceAll("^0+", ""));
                                endArea = Integer.parseInt(parts[1].replaceAll("^0+", "").isEmpty() ? "0" : parts[1].replaceAll("^0+", ""));
                            } else {
                                startArea = Integer.parseInt(gameArea.replaceAll("^0+", "").isEmpty() ? "0" : gameArea.replaceAll("^0+", ""));
                                endArea = startArea;
                            }
                            for (int i = startArea; i <= endArea; i++) {
                                String zoneNum = String.format("%02d", i);
                                String zoneName = "tx" + zoneNum;
                                String zoneDisplay = "原" + i + "区账号";
                                out.print("<option value=\"" + zoneName + "\">" + zoneDisplay + "</option>\n");
                            }
                        %>
                    </select>
                </div>

                <div class="form-group">
                    <label class="form-label form-label-required">用户名</label>
                    <input type="text" name="_user" value="<%=p_user%>" class="form-control" placeholder="请输入游戏账号" required>
                </div>

                <div class="form-group">
                    <label class="form-label form-label-required">密码</label>
                    <input type="password" name="_pswd" value="<%=p_pswd%>" class="form-control" placeholder="请输入密码" required>
                </div>

                <input type="hidden" name="m_key" value="<%=m_key%>">
                <input type="hidden" name="mid" value="<%=mid%>">

                <div class="login-buttons">
                    <button type="submit" class="btn btn-primary btn-login">登 录</button>
                    <button type="reset" class="btn btn-secondary">重 置</button>
                </div>
            </form>

            <div class="divider-text">
                <span>━━━━━━━ 还没有账号？ ━━━━━━━</span>
            </div>

            <div class="register-link">
                <p style="margin-bottom: 12px; color: #6B5B47;">立即创建账号，开启你的游戏之旅</p>
                <a href="javascript:void(0);" onclick="goRegister()" class="btn btn-secondary btn-block">
                    ✨ 新账号注册
                </a>
            </div>

            <div style="text-align: center; margin-top: 30px; padding-top: 20px; border-top: 1px solid rgba(0, 255, 255, 0.2);">
                <a href="https://www.wapmud.com" class="btn btn-secondary btn-block">
                    🏠 返回首页
                </a>
            </div>

            <script>
                function goRegister() {
                    var gamePre = document.getElementById('game_pre').value;
                    var m_key = '<%=m_key%>';
                    var mid = '<%=mid%>';
                    window.location.href = './regnew.jsp?_game_pre=' + encodeURIComponent(gamePre) + '&m_key=' + encodeURIComponent(m_key) + '&mid=' + encodeURIComponent(mid);
                }
            </script>
        </div>
    </div>

    <div class="footer">
        <div class="footer-text">
            © 2025 天下AI网游
        </div>
    </div>

<!-- 界面选择脚本 -->
<script>
    // 检查是否有保存的界面选择
    function checkSavedUI() {
        var savedUI = localStorage.getItem('mud_ui_choice');
        var savedTime = localStorage.getItem('mud_ui_choice_time');

        // 如果有保存的选择且在30天内，直接跳转
        if (savedUI && savedTime) {
            var daysSince = (Date.now() - parseInt(savedTime)) / (1000 * 60 * 60 * 24);
            if (daysSince < 30) {
                if (savedUI === 'new') {
                    window.location.href = 'web_vue/index.html';
                    return true;
                } else if (savedUI === 'old') {
                    document.getElementById('uiSelection').style.display = 'none';
                    document.getElementById('oldLoginForm').style.display = 'block';
                    return true;
                }
            }
        }

        // 检查 URL 参数
        var urlParams = new URLSearchParams(window.location.search);
        var uiParam = urlParams.get('ui');
        if (uiParam === 'back') {
            // 强制显示选择页，清除保存的选择
            localStorage.removeItem('mud_ui_choice');
            localStorage.removeItem('mud_ui_choice_time');
            return false;
        }

        return false;
    }

    // 选择界面
    function selectUI(ui) {
        var remember = document.getElementById('rememberChoice').checked;

        if (ui === 'new') {
            if (remember) {
                localStorage.setItem('mud_ui_choice', 'new');
                localStorage.setItem('mud_ui_choice_time', String(Date.now()));
            } else {
                localStorage.removeItem('mud_ui_choice');
                localStorage.removeItem('mud_ui_choice_time');
            }
            window.location.href = 'web_vue/index.html';
        } else if (ui === 'old') {
            if (remember) {
                localStorage.setItem('mud_ui_choice', 'old');
                localStorage.setItem('mud_ui_choice_time', String(Date.now()));
            } else {
                localStorage.removeItem('mud_ui_choice');
                localStorage.removeItem('mud_ui_choice_time');
            }
            document.getElementById('uiSelection').style.display = 'none';
            document.getElementById('oldLoginForm').style.display = 'block';
        }
    }

    // 显示界面选择
    function showUISelection() {
        document.getElementById('oldLoginForm').style.display = 'none';
        document.getElementById('uiSelection').style.display = 'block';
    }

    // 页面加载时检查
    window.onload = function() {
        checkSavedUI();
    };
</script>

<!-- 翻译插件初始化 (Translate.js Integration) -->
<div class="translate-area">
  <div class="translate-wrapper">
    <script src="includes/translate.js"></script>
    <script>
      translate.language.setLocal('chinese_simplified');
      translate.service.use('client.edge');
      translate.setAutoDiscriminateLocalLanguage();
      translate.execute();
    </script>
  </div>
</div>

<style>
.translate-area {
  width: 100%;
  height: 50px;
  display: flex;
  justify-content: center;
  align-items: center;
  background: transparent;
  margin-top: 20px;
}
.translate-wrapper {
  max-width: 200px;
  padding: 5px;
  font-size: 11px;
  color: #888;
  text-align: center;
}
.translate-wrapper select {
  font-size: 11px;
  padding: 2px 4px;
  max-width: 180px;
}
</style>

</body>
</html>