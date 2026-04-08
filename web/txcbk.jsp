<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8" import="java.text.SimpleDateFormat,java.util.Date,java.net.*,java.util.*,java.io.*,java.util.concurrent.*" %>
<%@include file="common.inc"%>
<!DOCTYPE html>
<html>
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
	<meta name="viewport" content="maximum-scale=1.0,minimum-scale=1.0,user-scalable=0,width=device-width,initial-scale=1.0"/>
<title>手动充值-天下原版一区</title>
<link href="includes/intro.css" rel="stylesheet" type="text/css"/>
    <%=favicon%>
</head>
<body>

<div>
	<!DOCTYPE html>
	<html lang="zh-cn">
		<head>
			<meta charset="UTF-8">
			<meta name="viewport" content="maximum-scale=1.0,minimum-scale=1.0,user-scalable=0,width=device-width,initial-scale=1.0"/>
			<title>手动充值-天下原版一区</title>
			<link href="includes/bootstrap.min.css" rel="stylesheet">
			<style>
				body {
					font: Normal 18px "Arial Black";
					text-align: center;
				}
				@media (min-width: 768px) {
					body {
						margin: 0 auto;
						width: 414px;
					}
				}
				.logo {
					margin-top: 5px;
					width: 160px;
					height: 87px;
				}
				.form-group{
					width:90%;
					margin:5%;
				}
				.lable{
					width:60px;
					font-size: 16px;
					font-weight: 500;
					text-align: center;
				}
				.input{
					width:calc(100% - 80px);
					font-size: 16px;
				}
				.desc{
					width: 90%;
					margin: 5%;
				}
				.copyright{
					margin-top:30px;
				}
				.btns{
					margin-top:30px;
				}
				.btn{
					margin: 0 20px;
				}
				h2{
					font-size:20px;
				}
			</style>
		    <%=favicon%>
</head>
	<body>

<%
	// 防刷逻辑必须在任何 HTML 输出之前执行（包括 <img> 标签）
	// 这样才能在需要时正确调用 response.sendRedirect()
 ///////////////////////////////////////////////////////////////////////////
    // 使用内存限流机制防止 IP 短时间内重复充值
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
		response.sendRedirect("ip_exists.jsp"); // 重定向到提示页面
		return; // 重要：防止后续代码执行
	}
	///////////////////////////////////////////////////////////////////////////
%>

			<img src="logo.jpg" class="logo">


<%
	



//System.out.println("------txcbk.jsp 1\n");
  String user = request.getParameter("usid");
  String pswd = request.getParameter("pswd");
  String txd = request.getParameter("txd");
  if( user==null || pswd==null ||txd==null)
  	response.sendRedirect("./txadd.jsp?err=1");
  else{
	user = user.trim();
	pswd = pswd.trim();
	txd = txd.trim();
	if(user.length() == 0 || pswd.length() == 0 || txd.length() == 0)
  		response.sendRedirect("./txadd.jsp?err=1");//字符为空
	else if( user.length()<2 || pswd.length()<2 || txd.length()<2)
  		response.sendRedirect("./txadd.jsp?err=2");//字符小于2位
	//else if( user.length()>13 || pswd.length()>13 || txd.length()>13)
  	//	response.sendRedirect("./txadd.jsp?err=3");//字符大于10位
	else if( !pswd.equals("tx2023ksofe94") )
  		response.sendRedirect("./txadd.jsp?err=5");//管理密码错误
	else{
//		String user_pswd = user + pswd + txd;
//		if(!isLegalChar(user_pswd))
 // 			response.sendRedirect("./txadd.jsp?err=4");//非法字符
//		else
		{
//System.out.println("------txcbk.jsp 2\n");
	        String result = "";
			//send(writer1,("login_entrycheck_p "+projname+" "+area+""+user+" "+pswd+" "+sid2).getBytes());
	        //result = exec_pike("/usr/local/games/tx/szx_add.pike",user,txd);
			System.out.println("------txcbk.jsp exec_pike result=["+result+"]\n");
			//pike被调用返回的是main里面write的内容：
			//txcbk.jsp exec_pike result=[---------玩家账号---------fh012022020801---------购买金额---------1000]
			String resultData = "<h2 class=\"text-success\">充值成功</h2>";
			resultData += "<h4 class=\"desc\">帐号："+user+"</h4>";
			resultData += "<h4 class=\"desc\">天下点："+txd+"</h4>";
			out.println(resultData);
			/*
			if(result.equals("success")){
				String resultData = "<h2 class=\"text-success\">充值成功</h2>";
				resultData += "<h4 class=\"desc\">帐号："+user+"</h4>";
				resultData += "<h4 class=\"desc\">天下点："+txd+"</h4>";
				out.println(resultData);
			}else{
				String resultData = "<h2 class=\"text-danger\">充值失败！</h2>";
				resultData += "<h3 class=\"text-danger\">"+result+"</h3>";
				resultData += "<h5 class=\"desc\">帐号："+user+"</h4>";
				resultData += "<h5 class=\"desc\">天下点："+txd+"</h4>";
				out.println(resultData);
			}
			*/
		}
	}
  }
%>
		<br/>
		<h4 class="copyright">© 2022 天下原版一区<br/><br/><a href="./txadd.jsp">返回充值</a></h4>
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
			<script src="includes/jquery.min.js"></script>
			<script src="includes/bootstrap.min.js"></script>
			</html>

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
