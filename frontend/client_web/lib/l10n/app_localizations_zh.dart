// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get appTitle => 'FunnyX';

  @override
  String get login => '登录';

  @override
  String get register => '注册';

  @override
  String get username => '账号';

  @override
  String get password => '密码';

  @override
  String get home => '首页';

  @override
  String get shop => '商店';

  @override
  String get logout => '登出';

  @override
  String get loginHint => '密码登录、注册或合作方 OAuth。会话接口需要 Token Server 令牌。';

  @override
  String get registerOauthHint =>
      '先在 FunnyX 注册（路径 A）。合作方可使用平台 OAuth 让您在游戏内登录。若已有合作方账号，可使用合作方 OAuth（路径 B）。';

  @override
  String get noAccount => '没有账号？注册';

  @override
  String get haveAccount => '已有账号？登录';

  @override
  String get oauthCompareTitle => 'OAuth 2.0 — 两种方向';

  @override
  String get oauthPathBTitle => '路径 B — 合作方 → 平台';

  @override
  String get oauthPathBBody =>
      '您已在合作方游戏/网站注册。Client Web 发起合作方 OAuth；Session Token Server 交换 code/userinfo 并签发 FunnyX 会话（sts_…）。';

  @override
  String get oauthPathCTitle => '路径 C — 平台 → 合作方';

  @override
  String get oauthPathCBody =>
      '您先在 Client Web 注册。合作方跳转到 FunnyX 平台 OAuth（/v1/oauth/authorize）。授权后合作方用 code 在其游戏内登录您。';

  @override
  String get oauthContinuePartner => '使用合作方 OAuth 继续';

  @override
  String get oauthPlatformAuthorizeHint => '合作方嵌入的授权 URL（路径 C 演示）：';

  @override
  String get oauthCopyAuthorize => '复制授权 URL';

  @override
  String get oauthCopied => '已复制';

  @override
  String get oauthCompleting => '正在完成合作方 OAuth…';

  @override
  String get oauthSuccess => '合作方 OAuth 登录完成';

  @override
  String get oauthFailed => '合作方 OAuth 失败';

  @override
  String get linkPartnerAccount => '绑定合作方账号（OAuth）';

  @override
  String get shopCart => '购物车';

  @override
  String get shopOrders => '订单';

  @override
  String get shopCheckout => '结账';

  @override
  String get shopPayment => '支付';

  @override
  String get shopRetry => '重试';

  @override
  String get shopEmptyCatalog => '暂无商品。';

  @override
  String shopBaseCurrency(String currency) {
    return '标价币种 $currency';
  }

  @override
  String get shopSellerPlatform => '平台';

  @override
  String get shopSellerCorp => '企业';

  @override
  String get shopAddedToCart => '已加入购物车';

  @override
  String get shopCartEmpty => '购物车为空。';

  @override
  String get shopColItem => '商品';

  @override
  String get shopColUnitPrice => '单价';

  @override
  String get shopColQty => '数量';

  @override
  String get shopColLineTotal => '小计';

  @override
  String get shopColActions => '操作';

  @override
  String get shopColOrderId => '订单';

  @override
  String get shopColSeller => '卖家';

  @override
  String get shopColFiat => '法币';

  @override
  String get shopColStatus => '状态';

  @override
  String get shopColExpires => '过期';

  @override
  String shopSubtotal(String amount) {
    return '合计：$amount';
  }

  @override
  String get shopSelectGameAccount => '选择用于到账的游戏账号';

  @override
  String get shopGameAccount => '游戏账号';

  @override
  String get shopNoGameAccount => '没有可用的游戏账号绑定。请先绑定游戏账号（首页或合作方 OAuth）。';

  @override
  String get shopOrderSummary => '订单摘要';

  @override
  String get shopPlaceOrder => '下单';

  @override
  String get shopPaymentHint => '待支付订单保留 24 小时。请通过合作方收银台付款，然后刷新状态。';

  @override
  String get shopNoPendingOrders => '没有待支付订单。';

  @override
  String get shopNoOrders => '暂无订单。';

  @override
  String get shopPay => '去支付';

  @override
  String get shopCancel => '取消';

  @override
  String get shopCancelled => '订单已取消';

  @override
  String shopOrderDetail(int id) {
    return '订单 #$id';
  }

  @override
  String get shopOrderNotFound => '订单不存在';

  @override
  String get shopCredit => '到账';

  @override
  String get shopCreatedAt => '创建时间';

  @override
  String get shopPartnerOrder => '合作方单号';
}

/// The translations for Chinese, as used in China (`zh_CN`).
class AppLocalizationsZhCn extends AppLocalizationsZh {
  AppLocalizationsZhCn() : super('zh_CN');

  @override
  String get appTitle => 'FunnyX';

  @override
  String get login => '登录';

  @override
  String get register => '注册';

  @override
  String get username => '账号';

  @override
  String get password => '密码';

  @override
  String get home => '首页';

  @override
  String get shop => '商店';

  @override
  String get logout => '登出';

  @override
  String get loginHint => '密码登录、注册或合作方 OAuth。会话接口需要 Token Server 令牌。';

  @override
  String get registerOauthHint =>
      '先在 FunnyX 注册（路径 A）。合作方可使用平台 OAuth 让您在游戏内登录。若已有合作方账号，可使用合作方 OAuth（路径 B）。';

  @override
  String get noAccount => '没有账号？注册';

  @override
  String get haveAccount => '已有账号？登录';

  @override
  String get oauthCompareTitle => 'OAuth 2.0 — 两种方向';

  @override
  String get oauthPathBTitle => '路径 B — 合作方 → 平台';

  @override
  String get oauthPathBBody =>
      '您已在合作方游戏/网站注册。Client Web 发起合作方 OAuth；Session Token Server 交换 code/userinfo 并签发 FunnyX 会话（sts_…）。';

  @override
  String get oauthPathCTitle => '路径 C — 平台 → 合作方';

  @override
  String get oauthPathCBody =>
      '您先在 Client Web 注册。合作方跳转到 FunnyX 平台 OAuth（/v1/oauth/authorize）。授权后合作方用 code 在其游戏内登录您。';

  @override
  String get oauthContinuePartner => '使用合作方 OAuth 继续';

  @override
  String get oauthPlatformAuthorizeHint => '合作方嵌入的授权 URL（路径 C 演示）：';

  @override
  String get oauthCopyAuthorize => '复制授权 URL';

  @override
  String get oauthCopied => '已复制';

  @override
  String get oauthCompleting => '正在完成合作方 OAuth…';

  @override
  String get oauthSuccess => '合作方 OAuth 登录完成';

  @override
  String get oauthFailed => '合作方 OAuth 失败';

  @override
  String get linkPartnerAccount => '绑定合作方账号（OAuth）';

  @override
  String get shopCart => '购物车';

  @override
  String get shopOrders => '订单';

  @override
  String get shopCheckout => '结账';

  @override
  String get shopPayment => '支付';

  @override
  String get shopRetry => '重试';

  @override
  String get shopEmptyCatalog => '暂无商品。';

  @override
  String shopBaseCurrency(String currency) {
    return '标价币种 $currency';
  }

  @override
  String get shopSellerPlatform => '平台';

  @override
  String get shopSellerCorp => '企业';

  @override
  String get shopAddedToCart => '已加入购物车';

  @override
  String get shopCartEmpty => '购物车为空。';

  @override
  String get shopColItem => '商品';

  @override
  String get shopColUnitPrice => '单价';

  @override
  String get shopColQty => '数量';

  @override
  String get shopColLineTotal => '小计';

  @override
  String get shopColActions => '操作';

  @override
  String get shopColOrderId => '订单';

  @override
  String get shopColSeller => '卖家';

  @override
  String get shopColFiat => '法币';

  @override
  String get shopColStatus => '状态';

  @override
  String get shopColExpires => '过期';

  @override
  String shopSubtotal(String amount) {
    return '合计：$amount';
  }

  @override
  String get shopSelectGameAccount => '选择用于到账的游戏账号';

  @override
  String get shopGameAccount => '游戏账号';

  @override
  String get shopNoGameAccount => '没有可用的游戏账号绑定。请先绑定游戏账号（首页或合作方 OAuth）。';

  @override
  String get shopOrderSummary => '订单摘要';

  @override
  String get shopPlaceOrder => '下单';

  @override
  String get shopPaymentHint => '待支付订单保留 24 小时。请通过合作方收银台付款，然后刷新状态。';

  @override
  String get shopNoPendingOrders => '没有待支付订单。';

  @override
  String get shopNoOrders => '暂无订单。';

  @override
  String get shopPay => '去支付';

  @override
  String get shopCancel => '取消';

  @override
  String get shopCancelled => '订单已取消';

  @override
  String shopOrderDetail(int id) {
    return '订单 #$id';
  }

  @override
  String get shopOrderNotFound => '订单不存在';

  @override
  String get shopCredit => '到账';

  @override
  String get shopCreatedAt => '创建时间';

  @override
  String get shopPartnerOrder => '合作方单号';
}

/// The translations for Chinese, as used in Taiwan (`zh_TW`).
class AppLocalizationsZhTw extends AppLocalizationsZh {
  AppLocalizationsZhTw() : super('zh_TW');

  @override
  String get appTitle => 'FunnyX';

  @override
  String get login => '登入';

  @override
  String get register => '註冊';

  @override
  String get username => '帳號';

  @override
  String get password => '密碼';

  @override
  String get home => '首頁';

  @override
  String get shop => '商店';

  @override
  String get logout => '登出';

  @override
  String get loginHint => '密碼登入、註冊或合作方 OAuth。會話接口需要 Token Server 權杖。';

  @override
  String get registerOauthHint =>
      '先在 FunnyX 註冊（路徑 A）。合作方可使用平台 OAuth 让您在遊戲内登入。若已有合作方帳號，可使用合作方 OAuth（路徑 B）。';

  @override
  String get noAccount => '沒有帳號？註冊';

  @override
  String get haveAccount => '已有帳號？登入';

  @override
  String get oauthCompareTitle => 'OAuth 2.0 — 兩種方向';

  @override
  String get oauthPathBTitle => '路徑 B — 合作方 → 平台';

  @override
  String get oauthPathBBody =>
      '您已在合作方遊戲/網站註冊。Client Web 發起合作方 OAuth；Session Token Server 交換 code/userinfo 并簽發 FunnyX 會話（sts_…）。';

  @override
  String get oauthPathCTitle => '路徑 C — 平台 → 合作方';

  @override
  String get oauthPathCBody =>
      '您先在 Client Web 註冊。合作方跳轉到 FunnyX 平台 OAuth（/v1/oauth/authorize）。授權后合作方用 code 在其遊戲内登入您。';

  @override
  String get oauthContinuePartner => '使用合作方 OAuth 继续';

  @override
  String get oauthPlatformAuthorizeHint => '合作方嵌入的授權 URL（路徑 C 演示）：';

  @override
  String get oauthCopyAuthorize => '複製授權 URL';

  @override
  String get oauthCopied => '已複製';

  @override
  String get oauthCompleting => '正在完成合作方 OAuth…';

  @override
  String get oauthSuccess => '合作方 OAuth 登入完成';

  @override
  String get oauthFailed => '合作方 OAuth 失敗';

  @override
  String get linkPartnerAccount => '綁定合作方帳號（OAuth）';

  @override
  String get shopCart => '購物車';

  @override
  String get shopOrders => '訂單';

  @override
  String get shopCheckout => '結帳';

  @override
  String get shopPayment => '支付';

  @override
  String get shopRetry => '重試';

  @override
  String get shopEmptyCatalog => '暫無商品。';

  @override
  String shopBaseCurrency(String currency) {
    return '標價幣種 $currency';
  }

  @override
  String get shopSellerPlatform => '平台';

  @override
  String get shopSellerCorp => '企業';

  @override
  String get shopAddedToCart => '已加入購物車';

  @override
  String get shopCartEmpty => '購物車為空。';

  @override
  String get shopColItem => '商品';

  @override
  String get shopColUnitPrice => '單價';

  @override
  String get shopColQty => '數量';

  @override
  String get shopColLineTotal => '小計';

  @override
  String get shopColActions => '操作';

  @override
  String get shopColOrderId => '訂單';

  @override
  String get shopColSeller => '賣家';

  @override
  String get shopColFiat => '法幣';

  @override
  String get shopColStatus => '狀態';

  @override
  String get shopColExpires => '過期';

  @override
  String shopSubtotal(String amount) {
    return '合計：$amount';
  }

  @override
  String get shopSelectGameAccount => '选择用于到帳的遊戲帳號';

  @override
  String get shopGameAccount => '遊戲帳號';

  @override
  String get shopNoGameAccount => '沒有可用的遊戲帳號綁定。请先綁定遊戲帳號（首頁或合作方 OAuth）。';

  @override
  String get shopOrderSummary => '訂單摘要';

  @override
  String get shopPlaceOrder => '下單';

  @override
  String get shopPaymentHint => '待支付訂單保留 24 小时。请通过合作方收银台付款，然后刷新狀態。';

  @override
  String get shopNoPendingOrders => '沒有待支付訂單。';

  @override
  String get shopNoOrders => '暫無訂單。';

  @override
  String get shopPay => '去支付';

  @override
  String get shopCancel => '取消';

  @override
  String get shopCancelled => '訂單已取消';

  @override
  String shopOrderDetail(int id) {
    return '訂單 #$id';
  }

  @override
  String get shopOrderNotFound => '訂單不存在';

  @override
  String get shopCredit => '到帳';

  @override
  String get shopCreatedAt => '建立時間';

  @override
  String get shopPartnerOrder => '合作方單號';
}
