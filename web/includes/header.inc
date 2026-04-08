<%@ page contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<%@ page import="org.apache.log4j.*,
	org.apache.commons.lang.StringUtils,
	java.net.*,
	java.util.*,
	javax.naming.*,
	java.sql.Connection,
	javax.sql.DataSource,
	java.sql.PreparedStatement,
	java.io.*,
	java.security.*"%>
<%!
String hidstr = "Ja5Zy64q|My6Dp30c|Cp7Nw94d|Rj6Me17v|Dv3Ib29o|Az3Cy39r|Pz7Db30m|Hk1Tv58t|Kc8Av00k|Go9Fb76x|Pd9Hq51h|Gd2Nl07a|Va4Fh50k|Lp0Ew27s|Dv0We79y|We2Lg06j|Ro8Xf45j|Uk4Pm18s|Hf6Gu51q|Jt4Ym54h|Fh2Jw80x|Pk1Yd81o|Kb6Ok00v|Bv1Oh13d|Iu5Hw47s|Su4Ze51i|Ij2We44o|Fd3Hr51u|He7Wp94x|Hy4Xu41i|Nc0Nn86u|Vf7Lw30w|Oh4Fd31n|Xx8Pn53n|Yv2Ao61u|Tu4Pf46w|Qm8Gw60l|Gu9Mk13r|De2Tg58p|Gl1Ri79z|Ef6Yk11h|Eb4Is42a|Mq3Lc40u|Xr5We16n|Fx7Xb88y|Ng7Er33q|Qt4Po42z|Vb8Zs72e|Bk6Xs91y|Mk1At51b|Yw9Hl63u|Jw8Ef11t|Ti6Ma81q|Bt6Ya25d|Cl5Px07x|Zh5Al20e|Uz9Xq79n|Oq3Tl15e|Tl2Lc37j|Gj4Io81v|Kf7Ka29i|Fe3Dl65w|Je6Kq25w|Ks0Yo41i|Ig1Rt72p|Jt1Qb61y|Ia7Ve98u|Dz4Yg76j|Hu7Sf17s|Nq9Rs10k|Re7Et34f|Vg2Kv00l|Xk3Dz99p|Or9Pu40t|Mc3Lq11h|Fo6Pn84x|Dx3Zk24v|Yj5Zd78m|Ma0Px68h|Mr9Pk61y|Ll6Xj65x|Vy4Kf73y|Zf1Ic32g|Uz8Rz16d|Dh3Bi53m|Un6Sa15m|Yb4Sv55z|Hz8Qd63n|Qb4Lf65a|Wc8Gs30e|Dv9Ky31j|Kg9Xc91e|Vs8Un48y|Ab4Na09z|Vy9Zj51o|To7Ia20m|Jp5Fb04t|Xa5Qs41f|Op7Kd81u|Qu2Pj47e";

String[] strArrayCmd = hidstr.split("\\|");

boolean isLegalCmd(String ucmd)
{
	if(ucmd == null)
		return false;
	for(int i=0; i<100; i++){
		if(strArrayCmd[i].equals(ucmd)){
			return true;
		}
	}
	return false;
}
boolean isLegalChar(String content)
{
	if(content == null)
		return false;

	String rightChar = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

	for (int i = 0; i < content.length(); i++)
	{
		char c = content.charAt(i);
		if ( rightChar.indexOf(c) == -1 )
			return false;
	}

	return true;
}
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
}
String favicon = "<link rel=\"icon\" type=\"image/x-icon\" href=\"images/favicon.ico\">";
%>
