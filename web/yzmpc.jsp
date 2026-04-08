<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<!DOCTYPE html>
<html>
<head>
	
	<meta charset="UTF-8">
	<meta name="viewport" content="maximum-scale=1.0,minimum-scale=1.0,user-scalable=0,width=device-width,initial-scale=1.0"/>
<title>《天下无名网游》</title>
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
			var usid = document.getElementById("usid").value;
			var pswd = document.getElementById("pswd").value;
			window.location.replace("pg.jsp?usid="+usid+"&pswd="+pswd);
		}
	}
</script>

<body>
<%
  	String err = request.getParameter("err");
	if(err==null) err = "sucess";
%>
<div>
	<!DOCTYPE html>
	<html lang="zh-cn">
		<head>
			<meta charset="UTF-8">
			<meta name="viewport" content="maximum-scale=1.0,minimum-scale=1.0,user-scalable=0,width=device-width,initial-scale=1.0"/>
			<title>《天下无名网游》</title>
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

	<img src="../logo.gif" class="logo">
		<h2>原班团队，经典天下</h2>
		<h2>醇香文字，仙侠江湖</h2>

		<%if(err.equals("1")){%>
		<h4 class="text-danger">错误：用户名或密码不能为空</h4>	
		<%}else if(err.equals("2")){%>
		<h4 class="text-danger">错误：用户名或密码不能小于2位</h4>	
		<%}else if(err.equals("3")){%>
		<h4 class="text-danger">错误：用户名或密码不能大于13位</h4>	
		<%}else if(err.equals("4")){%>
		<h4 class="text-danger">错误：用户名或密码为非正常字符，只可以输入英文或数字</h4>	
        <%}else if(err.equals("5")){%>
        <h4 class="text-danger">错误：账号密码错误，请重试</h4>
        <%}else if(err.equals("101")){%>
        <h4 class="text-danger">错误：验证码错误，请重试</h4>
		<%}%>
			
<!--			
			<form action="pg.jsp" method="post">
				<div class="form-group">
					<input type="text"  class="form-control"  id="" name="usid" maxlength="16" placeholder="输入账号(不超过13位英文或数字)">
				</div>
				<div class="form-group">
					<input type="password" class="form-control" id="" name="pswd" maxlength="16" placeholder="输入密码(不超过13位英文或数字)">
				</div>
				<h4 class="desc">新账号直接登录即可注册</h4>
				<div class="btns">
					<button type="submit" class="btn btn-info btn-lg">登录游戏</button>
			</div>
			</form>

-->

				<div class="form-group">
					<input type="text"  class="form-control"  id="usid" name="usid" maxlength="16" placeholder="输入账号(不超过13位英文或数字)">
				</div>
				<div class="form-group">
					<input type="password" class="form-control" id="pswd" name="pswd" maxlength="16" placeholder="输入密码(不超过13位英文或数字)">
				</div>
			<h4 class="desc">新账号直接登录即可注册</h4>
			
			<div id="checkCode" class="code"  onclick="createCode(4)" ></div>
			<a onclick="createCode(4)">看不清换一个验证码</a>
			<div class="form-group">
				<input type="text" id="inputCode" />
				</div>
			请输入上面的四位验证码
				
				<div class="btns">
					<button type="button" class="btn btn-danger" onclick="validateCode()">登录游戏</button>
				</div>
				<!--
				<font style="color:Olive; font-size:large;">[color:Olive large]</font><br/>
				<font style="color:DimGrey; font-size:x-large;">[color:DimGrey]</font><br/>
				<font style="color:DarkBlue">[color:DarkBlue]</font><br/>
				<font style="color:DarkViolet">[color:DarkViolet]</font><br/>
				<font style="color:DARKORANGE; font-size:x-large;">[color:DARKORANGE]</font><br/>
				<font style="color:DarkGreen">[color:DarkGreen]</font><br/>
				-->
		<h5 class="text-Gray">[注：所有游戏为测试版本，均无充值付费接口]<br/></h5>
		<h5 class="text-danger">本游戏仅在非中国地区运营，请遵守本地法律使用本游戏服务<br/></h5>
		<h6 class="text-danger" style="color:DimGrey">版权所有<br/></h6>
		<h6 class="text-danger" style="color:DarkGreen" >Copyright © 2022, COOLIT, Co,. Ltd.<br/></h6>
		<h6 class="text-danger" style="color:DarkGreen"> All Rights Reserved. <br/></h6>
		<h6 class="text-danger" style="color:DimGrey">地点日本美国<br/></h6>
		<!--
		<h5 class="text-Gray">温馨提示：每个区账号都是独立的，互相不能共享使用<br/></h5>
		<h6 class="text-danger">本游戏仅在非中国地区运营，请遵守本地法律使用本游戏服务<br/></h6>
		<h4 class="copyright">© 2022 《天下无名网游》<br/>
			<h6 class="text-danger">地点日本美国<br/></h6>
		-->

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
