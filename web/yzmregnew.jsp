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

<script>
    //页面加载时，生成随机验证码
    window.onload=function(){
         createCode(4);    
    }

	//生成验证码的方法
	function createCode(length) {
		var code = "";
		var codeLength = parseInt(length); //验证码的长度
		var checkCode = document.getElementById("checkCode");
		var codeChars = new Array(0,1,2,3,4,5,6,7,8,9,'a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z'); 

		for (var i = 0; i < codeLength; i++){
			var charNum = Math.floor(Math.random() * 62);
			code += codeChars[charNum];
		}
		if (checkCode){
			checkCode.className = "code";
			checkCode.innerHTML = code;
		}
	}

	function validateCode(){
		var checkCode = document.getElementById("checkCode").innerHTML;
		var inputCode = document.getElementById("inputCode").value;
		console.log(checkCode);
		console.log(inputCode);
		if (inputCode.length <= 0){
			alert("请输入验证码！");
		}
		else if (inputCode.toUpperCase() != checkCode.toUpperCase()){
			alert("验证码输入有误！");
			createCode(4);
		}
		else{
			alert("验证码正确！");
			var usid = document.getElementById("_user").value;
			var pswd = document.getElementById("_pswd").value;
			window.location.replace("./login_reg.jsp?regnewFlag=1&_user="+usid+"&_pswd="+pswd);
		}
	}
</script>

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

<%	
String m_key = request.getParameter("m_key");
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
	else if("4".equals(error_str))
		out.print("<h4 class=\"text-danger\">友情提示：该游戏帐号已经有人使用，请换一个账号重试。 <br/></h4>");  
	else if("5".equals(error_str))
		out.print("<h4 class=\"text-danger\">友情提示：游戏账号和密码必须是2~12位的英文或者数字，或者两者的组合。 <br/></h4>");  
	else if("6".equals(error_str))
		out.print("<h4 class=\"text-danger\">友友情提示：您输入的用户名不存在，是否要注册这个帐户? <br/></h4>");  
	%>
		<h4 class="copyright">一区新用户注册</h4>
		<h6 class="text-danger">注：用户名和密码必须是2-12位之间，并且只能是数字和字母<br/></h6>
		
				<div class="form-group">
					<input type="text"  class="form-control"  id="_user" name="_user" maxlength="16" placeholder="输入账号(不超过13位英文或数字)">
				</div>
				<div class="form-group">
					<input type="password" class="form-control" id="_pswd" name="_pswd" maxlength="16" placeholder="输入密码(不超过13位英文或数字)">
				</div>
				
				<div id="checkCode" class="code"  onclick="createCode(4)" ></div>
					<a onclick="createCode(4)">看不清换一个验证码</a>
				<div class="form-group">
						<input type="text" id="inputCode" />
				</div>
				请输入上面的四位验证码

				<div class="btns">
					<button type="button" class="btn btn-success" onclick="validateCode()">确定提交</button>
				<br/>
				<br/>
				<a class="btn btn-info btn-lg" href="./auto.jsp?<%=paraStringESC%>" >自动注册</a>
				<br/>
				<a class="btn btn-danger" style="margin-top:10px;" href="./pc.jsp?<%=paraStringESC%>" >返回登录</a>
				</div>
					

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
