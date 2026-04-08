<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<%@include file="header1.inc"%>
<%@include file="common.inc"%>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <title>《天下AI网游》- 账号登录</title>
    <link href="includes/classical.css" rel="stylesheet" type="text/css"/>
    <style>
        .login-card {
            background: white;
            border-radius: 8px;
            box-shadow: 0 8px 24px rgba(44, 24, 16, 0.12);
            padding: 40px 30px;
            margin: 40px 0;
        }

        .zone-selector {
            margin: 30px 0;
        }

        .zone-selector-label {
            display: block;
            font-weight: 600;
            margin-bottom: 12px;
            color: #2C1810;
        }

        .login-form {
            display: flex;
            flex-direction: column;
            gap: 20px;
        }

        .input-group {
            position: relative;
        }

        .input-icon {
            position: absolute;
            left: 12px;
            top: 50%;
            transform: translateY(-50%);
            color: #D4AF37;
            font-size: 16px;
        }

        .form-control-with-icon {
            padding-left: 40px;
        }

        .login-buttons {
            display: flex;
            gap: 12px;
            margin-top: 20px;
        }

        .btn-login {
            flex: 1;
        }

        .error-message {
            background: rgba(167, 25, 48, 0.1);
            color: #A71930;
            padding: 12px 16px;
            border-radius: 4px;
            border-left: 4px solid #A71930;
            margin: 20px 0;
            font-size: 14px;
            animation: slideIn 0.3s ease-out;
        }

        .divider-text {
            color: #A89F93;
            font-size: 13px;
        }

        .register-link {
            text-align: center;
            margin-top: 20px;
            padding-top: 20px;
            border-top: 1px solid #D4C5B9;
        }

        .register-link a {
            color: #A71930;
            font-weight: 600;
            text-decoration: none;
            transition: all 0.3s ease;
        }

        .register-link a:hover {
            color: #D4AF37;
            transform: translateX(2px);
        }

        @media (max-width: 480px) {
            .login-card {
                padding: 24px 16px;
            }

            .login-buttons {
                flex-direction: column;
            }

            .btn {
                width: 100%;
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
            <div class="header-subtitle">原版天下 • 合服运营 • 诚邀各路英豪</div>
        </div>
    </div>

    <div class="container-narrow">
        <div class="login-card">
            <h2 style="text-align: center; margin-bottom: 30px;">账号登录</h2>

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
                <a href="./regnew.jsp?m_key=<%=m_key%>&mid=<%=mid%>" class="btn btn-secondary btn-block">
                    ✨ 新账号注册
                </a>
            </div>
        </div>
    </div>

    <div class="footer">
        <div class="footer-text">
            © 2025 天下AI网游 | 宇宙第一美观古典风设计<br>
        </div>
    </div>
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