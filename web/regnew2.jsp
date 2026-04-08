<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<%@include file="header1.inc"%>
<%@include file="common.inc"%>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <title>《天下AI网游》- 新账号注册</title>
    <link href="includes/classical.css" rel="stylesheet" type="text/css"/>
    <style>
        .register-card {
            background: white;
            border-radius: 8px;
            box-shadow: 0 8px 24px rgba(44, 24, 16, 0.12);
            padding: 40px 30px;
            margin: 40px 0;
        }

        .register-form {
            display: flex;
            flex-direction: column;
            gap: 20px;
        }

        .captcha-group {
            display: flex;
            gap: 12px;
            align-items: flex-end;
        }

        .captcha-input {
            flex: 1;
        }

        .captcha-button {
            flex: 0 0 100px;
            height: 40px;
            padding: 0;
            font-size: 12px;
        }

        .success-message {
            background: rgba(74, 124, 89, 0.1);
            color: #4A7C59;
            padding: 12px 16px;
            border-radius: 4px;
            border-left: 4px solid #4A7C59;
            margin: 20px 0;
            font-size: 14px;
            animation: slideIn 0.3s ease-out;
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

        .info-box {
            background: rgba(212, 175, 55, 0.05);
            border: 1px solid rgba(212, 175, 55, 0.3);
            border-radius: 4px;
            padding: 16px;
            margin: 20px 0;
            font-size: 13px;
            color: #6B5B47;
            line-height: 1.8;
        }

        .info-box strong {
            color: #A71930;
        }

        .register-buttons {
            display: flex;
            gap: 12px;
            margin-top: 20px;
        }

        .btn-register {
            flex: 1;
        }

        .login-link {
            text-align: center;
            margin-top: 20px;
            padding-top: 20px;
            border-top: 1px solid #D4C5B9;
        }

        .login-link a {
            color: #A71930;
            font-weight: 600;
            text-decoration: none;
            transition: all 0.3s ease;
        }

        .login-link a:hover {
            color: #D4AF37;
            transform: translateX(-2px);
        }

        @media (max-width: 480px) {
            .register-card {
                padding: 24px 16px;
            }

            .register-buttons,
            .captcha-group {
                flex-direction: column;
            }

            .btn,
            .captcha-input,
            .captcha-button {
                width: 100%;
            }

            .captcha-button {
                height: 40px;
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
            <div class="header-subtitle">创建新账号 • 开启冒险 • 诚邀各路英豪</div>
        </div>
    </div>

    <div class="container-narrow">
        <div class="register-card">
            <h2 style="text-align: center; margin-bottom: 30px;">✨ 新账号注册</h2>

            <%
                String error_str = request.getParameter("err");
                String success_str = request.getParameter("suc");

                if("1".equals(error_str))
                    out.print("<div class='error-message'>❌ 用户名和密码不能为空，请修改后重试</div>");
                else if("2".equals(error_str))
                    out.print("<div class='error-message'>❌ 用户名和密码不能少于2个字符，请修改后重试</div>");
                else if("3".equals(error_str))
                    out.print("<div class='error-message'>❌ 用户名和密码只能是大小写字母或数字，请修改后重试</div>");
                else if("4".equals(error_str))
                    out.print("<div class='error-message'>❌ 该账号已存在，请修改后重试</div>");
                else if("5".equals(error_str))
                    out.print("<div class='error-message'>❌ 验证码错误，请重新输入</div>");
                else if("1".equals(success_str))
                    out.print("<div class='success-message'>✅ 账号创建成功！请返回登录页进行登录</div>");
            %>

            <div class="info-box">
                <strong>📋 注册须知：</strong><br>
                • 账号和密码为<strong>2-12个</strong>英文或数字<br>
                • 区分大小写，请妥善保管<br>
                • 一个邮箱只能注册一个账号
            </div>

            <form action="./login_reg.jsp?regnewFlag=1" method="post" class="register-form">
                <div class="form-group">
                    <label class="form-label form-label-required">游戏账号</label>
                    <input type="text" name="_user" class="form-control" placeholder="2-12位英文或数字" required maxlength="12">
                </div>

                <div class="form-group">
                    <label class="form-label form-label-required">设置密码</label>
                    <input type="password" name="_pswd" class="form-control" placeholder="2-12位英文或数字" required maxlength="12">
                </div>

                <div class="form-group">
                    <label class="form-label form-label-required">确认密码</label>
                    <input type="password" name="_pswd_confirm" class="form-control" placeholder="请再输入一次密码" required maxlength="12">
                </div>

                <div class="form-group">
                    <label class="form-label form-label-required">验证码</label>
                    <div class="captcha-group">
                        <input type="text" name="code" class="form-control captcha-input" placeholder="请输入验证码" required maxlength="4">
                        <div class="captcha-button" id="checkCode" style="background: linear-gradient(135deg, #D4AF37 0%, #B8860B 100%); border-radius: 4px; display: flex; align-items: center; justify-content: center; color: white; font-weight: 600; cursor: pointer; user-select: none;">
                            刷新验证码
                        </div>
                    </div>
                </div>

                <div class="register-buttons">
                    <button type="submit" class="btn btn-primary btn-register">立即注册</button>
                    <button type="reset" class="btn btn-secondary">清空</button>
                </div>
            </form>

            <div class="login-link">
                <p style="margin-bottom: 12px; color: #6B5B47;">已有账号？返回登录</p>
                <a href="./pc.jsp" class="btn btn-secondary btn-block">
                    ← 返回登录
                </a>
            </div>
        </div>
    </div>

    <div class="footer">
        <div class="footer-text">
            © 2025 天下AI网游 | 宇宙第一美观古典风设计<br>
        </div>
    </div>

    <script>
        // 生成验证码
        function createCode(length) {
            var code = "";
            var codeLength = parseInt(length);
            var checkCode = document.getElementById("checkCode");
            var codeChars = new Array(0,1,2,3,4,5,6,7,8,9,'a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z');
            for (var i = 0; i < codeLength; i++){
                var charNum = Math.floor(Math.random() * 62);
                code += codeChars[charNum];
            }
            if (checkCode){
                checkCode.innerHTML = code;
                sessionStorage.setItem("checkCode", code);
            }
        }

        // 初始化验证码
        window.onload = function(){
            createCode(4);
        };

        // 点击刷新验证码
        document.getElementById("checkCode").addEventListener("click", function(){
            createCode(4);
        });
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