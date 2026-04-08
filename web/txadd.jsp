<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<!DOCTYPE html>
<html>
<head>
	
	<meta charset="UTF-8">
	<meta name="viewport" content="maximum-scale=1.0,minimum-scale=1.0,user-scalable=0,width=device-width,initial-scale=1.0"/>
<title>手动充值-天下原版一区</title>
<link href="includes/intro.css" rel="stylesheet" type="text/css"/>
    <%=favicon%>
</head>
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
			<img src="logo.jpg" class="logo">
			<h2>手动充值-天下原版一区</h2>

		<%if(err.equals("1")){%>
		<h4 class="text-danger">错误：用户id或天下点数额不能为空</h4>	
		<%}else if(err.equals("2")){%>
		<h4 class="text-danger">错误：用户id或天下点数额不能小于2位</h4>	
		<%}else if(err.equals("3")){%>
		<h4 class="text-danger">错误：用户id或天下点数额不能大于13位</h4>	
		<%}else if(err.equals("4")){%>
		<h4 class="text-danger">错误：用户id或天下点数额为非正常字符，只可以输入英文或数字</h4>	
		<%}else if(err.equals("5")){%>
		<h4 class="text-danger">错误：密码错误，请确认重新输入</h4>	
		<%}%>
			
			<form action="txcbk.jsp" method="post">
				<div class="form-group">
					<input type="text"  class="form-control"  id="" name="usid" maxlength="16" placeholder="输入用户id">
				</div>
				<div class="form-group">
					<input type="text" class="form-control" id="" name="txd" maxlength="16" placeholder="输入天下点数额">
				</div>
				<div class="form-group">
					<input type="password" class="form-control" id="" name="pswd" maxlength="16" placeholder="输入密码(不超过13位英文或数字)">
				</div>
				<div class="btns">
					<button type="submit" class="btn btn-danger">确定充值</button>
			</div>
			</form>
		<h5 class="text-danger">注：天下点数额将直接进入玩家账号中<br/></h5>
		<h4 class="copyright">© 2022 天下原版一区<br/><br/><a href="./pc.jsp">游戏首页</a></h4>
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
