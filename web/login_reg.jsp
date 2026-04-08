<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8" import="java.text.SimpleDateFormat,java.util.Date,java.net.*,java.util.*,java.io.*,java.util.concurrent.*" %>
<%@include file="header1.inc"%>
<!DOCTYPE html>
<html lang="zh-CN">
<head>

<%!
/**
 * IP 限流管理器 - 防止同一 IP 短时间内重复操作
 */
static class IPRateLimiter {
    private static IPRateLimiter instance = null;
    private ConcurrentHashMap<String, Long> ipRecords = new ConcurrentHashMap<>();
    private static final long RATE_LIMIT_INTERVAL = 30 * 1000; // 30 秒

    public static synchronized IPRateLimiter getInstance() {
        if (instance == null) {
            instance = new IPRateLimiter();
        }
        return instance;
    }

    public boolean isRateLimited(String ip) {
        if (ip == null || ip.isEmpty()) {
            return false;
        }
        Long lastTimestamp = ipRecords.get(ip);
        long currentTime = System.currentTimeMillis();
        if (lastTimestamp == null) {
            ipRecords.put(ip, currentTime);
            return false;
        }
        long timeDiff = currentTime - lastTimestamp;
        if (timeDiff < RATE_LIMIT_INTERVAL) {
            return true;
        } else {
            ipRecords.put(ip, currentTime);
            return false;
        }
    }
}
%>

    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <title>《天下AI网游》- 注册成功</title>
    <link href="includes/modern-gaming.css" rel="stylesheet" type="text/css"/>
    <style>
        .success-card {
            background: rgba(26, 26, 62, 0.8);
            border: 2px solid rgba(0, 255, 255, 0.3);
            border-radius: 20px;
            padding: 40px 30px;
            margin: 40px auto;
            max-width: 500px;
            text-align: center;
            box-shadow:
                0 0 30px rgba(0, 255, 255, 0.1),
                inset 0 0 60px rgba(0, 255, 255, 0.05);
            position: relative;
            overflow: hidden;
        }

        .success-card::before {
            content: '';
            position: absolute;
            top: -50%;
            left: -50%;
            width: 200%;
            height: 200%;
            background: radial-gradient(circle, rgba(0, 255, 255, 0.1) 0%, transparent 70%);
            animation: pulse-glow 4s ease-in-out infinite;
        }

        @keyframes pulse-glow {
            0%, 100% { transform: translate(-25%, -25%) scale(1); opacity: 0.5; }
            50% { transform: translate(-25%, -25%) scale(1.2); opacity: 0.8; }
        }

        .success-icon {
            font-size: 80px;
            margin-bottom: 20px;
            animation: bounce-in 0.6s ease-out;
        }

        @keyframes bounce-in {
            0% { transform: scale(0); opacity: 0; }
            50% { transform: scale(1.2); }
            100% { transform: scale(1); opacity: 1; }
        }

        .success-title {
            font-size: 28px;
            color: var(--success);
            margin-bottom: 30px;
            text-shadow: 0 0 20px rgba(0, 255, 170, 0.5);
        }

        .account-info {
            background: rgba(0, 0, 0, 0.3);
            border-radius: 12px;
            padding: 20px;
            margin: 20px 0;
            border: 1px solid rgba(0, 255, 255, 0.2);
        }

        .account-info p {
            font-size: 16px;
            color: var(--text-primary);
            margin: 10px 0;
            padding: 8px;
            border-bottom: 1px solid rgba(0, 255, 255, 0.1);
        }

        .account-info p:last-child {
            border-bottom: none;
        }

        .account-info strong {
            color: var(--primary-cyan);
            margin-right: 10px;
        }

        .notice-section {
            background: rgba(0, 255, 170, 0.1);
            border-radius: 12px;
            padding: 20px;
            margin: 20px 0;
            text-align: left;
        }

        .notice-section h4 {
            color: var(--success);
            font-size: 16px;
            margin-bottom: 15px;
            display: flex;
            align-items: center;
        }

        .notice-section h4::before {
            content: '📢';
            margin-right: 10px;
        }

        .notice-section h5 {
            color: var(--text-secondary);
            font-size: 14px;
            margin: 8px 0;
            padding-left: 30px;
        }

        .btn-enter {
            background: linear-gradient(135deg, var(--success) 0%, #00cc88 100%);
            color: #000;
            padding: 15px 40px;
            font-size: 18px;
            font-weight: 700;
            border-radius: 12px;
            margin: 30px 0;
            display: inline-block;
        }

        .btn-enter:hover {
            transform: translateY(-3px);
            box-shadow: 0 10px 30px rgba(0, 255, 170, 0.4);
        }

        .footer-info {
            margin-top: 40px;
            padding: 20px;
            border-top: 1px solid rgba(0, 255, 255, 0.2);
            color: var(--text-secondary);
            font-size: 12px;
        }

        .footer-info p {
            margin: 5px 0;
        }

        .back-link {
            color: var(--primary-cyan);
            text-decoration: none;
            display: inline-flex;
            align-items: center;
            margin-top: 15px;
            padding: 10px 20px;
            border: 1px solid rgba(0, 255, 255, 0.3);
            border-radius: 8px;
            transition: all 0.3s ease;
        }

        .back-link:hover {
            background: rgba(0, 255, 255, 0.1);
            border-color: var(--primary-cyan);
        }
    </style>
    <%=favicon%>
</head>
<body class="classical-pattern">

<%
	// 防刷逻辑必须在任何 HTML 输出之前执行（包括 <img> 标签）
	// 这样才能在需要时正确调用 response.sendRedirect()

	//String testua = (String)request.getHeader("User-Agent");
	//String _ip = (String)request.getRemoteAddr();
	//System.out.println("[tx/login_reg.jsp IP=["+_ip+"] UA = ["+testua+"] \n");

 ///////////////////////////////////////////////////////////////////////////
    // 使用内存限流机制防止 IP 短时间内重复注册
	// 获取真实IP（透过 Cloudflare/CDN）
	String userIP = request.getHeader("CF-Connecting-IP");
	if (userIP == null || userIP.isEmpty()) {
		userIP = request.getHeader("X-Forwarded-For");
		if (userIP != null && userIP.contains(",")) {
			userIP = userIP.split(",")[0].trim();
		}
	}
	if (userIP == null || userIP.isEmpty()) {
		userIP = request.getRemoteAddr();
	}
	IPRateLimiter rateLimiter = IPRateLimiter.getInstance();

	if (rateLimiter.isRateLimited(userIP)) {
		response.sendRedirect("./ip_exists.jsp"); // 重定向到提示页面
		return; // 重要：防止后续代码执行
	}
	///////////////////////////////////////////////////////////////////////////
%>

    <div class="header">
        <div class="container-narrow">
            <img src="logo-1.gif" class="logo">
            <div class="header-title">《天下AI网游》</div>
            <div class="header-subtitle">原版天下 • 醇香文字 • 邀你同行</div>
        </div>
    </div>

    <div class="container-narrow">
<%
game_pre = request.getParameter("_game_pre");//设置登陆的区的名字
// 防御性检查：如果 game_pre 为 null，使用默认值（来自环境变量或配置）
if(game_pre == null || game_pre.trim().isEmpty()) {
	game_pre = System.getenv("GAME_AREA");
	if(game_pre == null) {
		game_pre = "tx";  // 最后的默认值
	}
}
  String z = (String)request.getParameter("z");
if(z==null)
	z=(String)request.getSession().getAttribute("z");
else
	request.getSession().setAttribute("z",z);
String user = request.getParameter("_user");
String pswd = request.getParameter("_pswd");
String regnewFlag = (String)request.getParameter("regnewFlag");
String m_key = (String)request.getParameter("m_key");
String mid= (String)request.getParameter("mid");
String ap_info= (String)request.getParameter("ap_info");
String user_url = java.net.URLEncoder.encode(user,"UTF-8");
String pswd_url = java.net.URLEncoder.encode(pswd,"UTF-8");
if( user==null || pswd==null){
	//out.print("用户名和密码不能为空，请修改后重试。 <br/>");
	response.sendRedirect("./regnew.jsp?_user="+user+"&_pswd="+pswd+"&err=1&"+paraString);
}
else{
	user = user.trim();
	pswd = pswd.trim();
	if(user.length() == 0 || pswd.length() == 0){
		//	out.print("用户名和密码不能为空，请修改后重试。 <br/>");
		response.sendRedirect("./regnew.jsp?_user="+user+"&_pswd="+pswd+"&err=1&"+paraString);
	}
	else if( user.length()<2 || pswd.length()<2 ){
		//out.print("|"+user + "|:|" + pswd + "|<br/>");
		//out.print("为了你的安全，用户名和密码不能少于2个字符，请修改后重试。 <br/>");
		response.sendRedirect("./regnew.jsp?_user="+user+"&_pswd="+pswd+"&err=2&"+paraString);
	}
	else if( user.length()>11 || pswd.length()>11 ){
		response.sendRedirect("./regnew.jsp?_user="+user+"&_pswd="+pswd+"&err=5&"+paraString);
	}
	else{
		String user_pswd = user + pswd;
		if(!isLegalChar(user_pswd)){
			response.sendRedirect("./regnew.jsp?_user="+user_url+"&_pswd="+pswd_url+"&err=3&"+paraString);
		}
		else {
			Socket socket = new Socket(ip,port);
			InputStream reader = socket.getInputStream();
			OutputStream writer = socket.getOutputStream();

			String sid = (String)session.getId();

			//send(writer,("login_regnew "+projname+" "+user+" "+pswd+" "+sid+" "+game_pre+" "+m_key+" "+userip+" "+userua).getBytes());
			String user_new = game_pre+user;
			send(writer,("login_regnew "+projname+" "+user_new+" "+pswd+" "+sid).getBytes());
			send(writer,"flush_filter".getBytes());
			socket.shutdownOutput();

			String ret = read(reader,"utf-8");

			if(writer!=null) writer.close();
			if(reader!=null) reader.close();
			if(socket!=null) socket.close();

			if(ret.equals("error1")){
				//resultData += "游戏帐号已经有人使用！<br/>";
				response.sendRedirect("./regnew.jsp?_user="+user+"&_pswd="+pswd+"&err=4&"+paraString);
			}
			else if(ret.equals("error2")){
				response.sendRedirect("./regnew.jsp?_user="+user+"&_pswd="+pswd+"&err=5&"+paraString);
			}
			else{
%>
        <div class="success-card">
            <div class="success-icon">🎉</div>
            <h1 class="success-title">注册成功！</h1>

            <div class="account-info">
                <p><strong>帐号：</strong><%=user%></p>
                <p><strong>密码：</strong><%=pswd%></p>
            </div>

            <a href="./do.jsp?_user=<%=user_new%>&_pswd=<%=pswd%>&_mkey=<%=m_key%>" class="btn btn-enter">
                进入游戏 →
            </a>

            <div class="notice-section">
                <h4>最新通告</h4>
                <h5>🎁 捐赠送大礼！(详情参见游戏内说明)</h5>
                <h5>⚡ 日常技能任务开放，不用辛苦刷技能经验了！</h5>
            </div>
        </div>

        <div class="footer-info">
            <p>本游戏仅在非中国地区运营，请遵守本地法律使用本游戏服务</p>
            <p>© 2024 《天下AI网游》• 天下团队</p>
            <a href="./pc.jsp" class="back-link">← 返回首页</a>
        </div>
<%
			}
		}
	}
}
%>
    </div>

</body>
</html>
