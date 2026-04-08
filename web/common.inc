<%@ page contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<%@ page import="
	java.net.URL,
	java.net.HttpURLConnection,
	javax.naming.*,
	java.sql.Connection,
	javax.sql.DataSource,
	java.sql.PreparedStatement,
	java.io.BufferedReader,
	java.io.InputStream,
	java.io.InputStreamReader"
%>
<%!

String exec_pike(String ...args) {
	long set1 = System.currentTimeMillis();
	try {
		java.util.LinkedList li= new java.util.LinkedList(java.util.Arrays.asList(args));
		li.add(0,"/usr/local/bin/pike");
		ProcessBuilder builder = new ProcessBuilder(li);
		builder.redirectErrorStream(true);
		Process process = builder.start();
		InputStream ins = process.getInputStream();
		BufferedReader br = new BufferedReader(new InputStreamReader(ins,"GB2312"));
		String result = "";
		String line;
		while ((line = br.readLine()) != null) {
			result += line;
		}
		// int code = process.exitValue();
		//long cost = System.currentTimeMillis() - set1;
		//loggerCommon.info("[exec_pike:" + StringUtils.join(args,' ') +  "] [result:" + result + "] [" + cost + "ms]");
		return result;
	} catch (Throwable e) {
		//loggerCommon.error("[exec_pike:" + StringUtils.join(args,' ') +  "] [error] [" + (System.currentTimeMillis() - set1) + "ms]",e);
		return "0";
	}
}

public String invoke_url(String url,String uid,String point) {
	long set1 = System.currentTimeMillis();
	String dest = url + "?uid="+uid+"&point="+point;
	try {
		URL myurl = new URL(dest);
		HttpURLConnection urlconn = (HttpURLConnection) myurl.openConnection();
		urlconn.setConnectTimeout(10000);
		urlconn.setReadTimeout(30000);
		urlconn.setDoOutput(true);
		urlconn.setRequestMethod("GET");
		urlconn.connect();
		InputStream ins = urlconn.getInputStream();
		BufferedReader br = new BufferedReader(new InputStreamReader(ins));
		String result = "";
		String line;
		while ((line = br.readLine()) != null) {
			result += line;
		}
		urlconn.disconnect();
		//long cost = System.currentTimeMillis() - set1;
		//loggerCommon.debug("[invoke_url " + dest + "] [result:" + result + "] [" + cost + "ms]");
		return result;
	} catch (Throwable e) {
		//loggerCommon.error("[invoike_url:" + dest + "] [error] [" + (System.currentTimeMillis() - set1) + 
		//		"ms]", e);
		return "0";
	}
}

boolean isComputer(HttpServletRequest request)
{
	return false;
}%>
