<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<%@ page import="
	java.net.URL,
	java.net.HttpURLConnection,
	javax.naming.*,
	java.sql.Connection,
	javax.sql.DataSource,
	java.sql.PreparedStatement,
	java.io.*,
	java.time.*,
	java.text.SimpleDateFormat,
	java.util.Date,
	java.util.Calendar,
	java.time.DayOfWeek"
%>
<%!
String[] paraList = {"z","mid","m_key"};

String ip="127.0.0.1";
int port=5555;
String projname="gamenv";
// game_pre 从环境变量动态读取，如果未设置则默认为 tx01
String game_pre = System.getenv("GAME_AREA") != null && !System.getenv("GAME_AREA").isEmpty()
    ? (System.getenv("GAME_AREA").startsWith("tx") ? System.getenv("GAME_AREA") : "tx" + System.getenv("GAME_AREA"))
    : "tx01";
String gamename_cn="原版天下AI网游";

String queryParameterStringESC(HttpServletRequest request)
{
	StringBuffer sb = new StringBuffer();

	boolean havePre= false;
	for(int i=0; i < paraList.length;i++)
	{
		String name = paraList[i];
		String value = request.getParameter(name);
		if(value != null)
		{
			if(havePre)
				sb.append("&amp;");
			else 
				havePre =true;

			sb.append(name);
			sb.append("=");
			sb.append(value);
		}
	}
	return sb.toString();
}

String queryParameterString(HttpServletRequest request)
{
	StringBuffer sb = new StringBuffer();

	boolean havePre= false;
	for(int i=0; i < paraList.length;i++)
	{
		String name = paraList[i];
		String value = request.getParameter(name);
		if(value != null)
		{
			if(havePre)
				sb.append("&");
			else 
				havePre =true;
			sb.append(name);
			sb.append("=");
			sb.append(value);
		}
	}
	return sb.toString();
}

public int log_spread_info(String m_key,String mid,String game_id,String user_id,String ap_info)
{
	return 0;
}

boolean isLegalChar(String content)
{
	if(content == null)
		return false;

	String rightChar = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

	for (int i = 0; i < content.length(); i++)
	{
		char c = content.charAt(i);//字�?�串�?的字�?
		if ( rightChar.indexOf(c) == -1 )
			return false;
	}

	return true;
}

//以下函数用于sockt读写
void send(OutputStream  writer,byte bytes[]) throws IOException
{
	writer.write(bytes);
	writer.write("\n".getBytes());
	writer.flush();
}

String read(InputStream reader,String charset) throws IOException
{
	BufferedReader r = new BufferedReader(new InputStreamReader(reader,charset));
	String s = "";
	String ret ="";
	int n;
	try
	{
		char buff[]=new char[4096];
		n=r.read(buff,0,4096);
		while(n!=-1)
		{
			ret+=new String(buff,0,n);
			n=r.read(buff,0,4096);
		}
	}
	catch(Exception e)
	{
		e.printStackTrace();
	}
	return ret;
}
%>
<%
try
{
    request.setCharacterEncoding("utf-8");
    }catch(Exception uae)
    {
        //com.lj.bbs.tools.setDefaultCharSet(request);
	}
	String mobile=request.getHeader("ia");
	String paraString=queryParameterString(request);
	String paraStringESC=queryParameterStringESC(request);
	String favicon = "<link rel=\"icon\" type=\"image/x-icon\" href=\"images/favicon.ico\">";
%>
