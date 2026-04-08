<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<%@include file="header1.inc"%>
<%@include file="common.inc"%>
<!DOCTYPE html>
<html>
<head>
	
	
	
	<meta charset="UTF-8">
	<meta name="viewport" content="maximum-scale=1.0,minimum-scale=1.0,user-scalable=0,width=device-width,initial-scale=1.0"/>
<title>《天下原版网游》[一区]</title>
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
			<title>《天下原版网游》[一区]</title>
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
					width: 240px;
					height: 64px;
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

	<img src="logo.gif" class="logo">
		<h2>原班团队，经典原版</h2>
		<h2>醇香文字，仙侠天下</h2>
		
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
		out.print("<h4 class=\"text-danger\">友情提示：用户名和密码不能为空，请修改后重试。 <br/></h4>");  
	else if("2".equals(error_str))
		out.print("<h4 class=\"text-danger\">友情提示：为了你的安全，用户名和密码不能少于2个字符，请修改后重试。 <br/></h4>");
	else if("3".equals(error_str))
		out.print("<h4 class=\"text-danger\">友情提示：用户名和密码只能是大小写字母或数字，请修改后重试。 <br/></h4>");  
	else if("5".equals(error_str))
		out.print("<h4 class=\"text-danger\">友情提示：游戏账号和密码必须是2~12位的英文或者数字，或者两者的组合。 <br/></h4>");  
	else if("4".equals(error_str))
		out.print("<h4 class=\"text-danger\">友情提示：您输入的用户名和密码认证失败或有人正在使用该帐号。 <br/></h4>");  
	else if("6".equals(error_str))
		out.print("<h4 class=\"text-danger\">友情提示：您输入的用户名和密码认证失败，是否需要找回密码？ <br/></h4>");  
	else if("7".equals(error_str))
		out.print("<h4 class=\"text-danger\">友情提示：系统犯晕了，请通知管理员。 <br/></h4>");  
%>
			<form action="./entrycheck.jsp?regnewFlag=0>" method="post">
				<div class="form-group">
					<input type="text"  class="form-control"  id="" name="_user" maxlength="16" placeholder="输入账号(不超过13位英文或数字)">
				</div>
				<div class="form-group">
					<input type="password" class="form-control" id="" name="_pswd" maxlength="16" placeholder="输入密码(不超过13位英文或数字)">
				</div>
				<div class="btns">
					<button type="submit" class="btn btn-info btn-lg">登录游戏</button>
					<br>
					<a class="btn btn-danger" style="margin-top:10px;" href="./regnew.jsp?<%=paraStringESC%>" >注册账号</a>
			</div>
			</form>
		<h4 class="copyright">一区</h4>
		<h5 class="text-Gray">温馨提示：每个区账号都是独立的，互相不能共享使用<br/></h5>
		<h6 class="text-danger">本游戏仅在非中国地区运营，请遵守本地法律使用本游戏服务<br/></h6>
		<h6 class="text-danger">版权所有 2022  天下团队 地点日本美国<br/></h6>
		<h4 class="copyright">© 2022 《天下原版网游》<br/><br/><a href="https://www.txgame.org">游戏首页</a></h4>
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
