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
  String get wallet => '钱包';

  @override
  String get walletProfile => '资料';

  @override
  String get walletEmail => '邮箱';

  @override
  String get walletHint => '商店结算或充值/提现回调后余额会更新。请刷新重新加载。';

  @override
  String get walletBalances => '余额';

  @override
  String get walletTransactions => '交易记录';

  @override
  String get walletEmptyBalances => '暂无钱包余额。';

  @override
  String get walletEmptyTxns => '暂无资金交易。';

  @override
  String get walletAvailable => '可用';

  @override
  String get walletLocked => '锁定';

  @override
  String get walletGameCoin => '游戏币';

  @override
  String get walletAmount => '数量';

  @override
  String get walletDeposit => '充值';

  @override
  String get walletWithdraw => '提现';

  @override
  String get walletTxnId => '单号';

  @override
  String get walletTxnType => '类型';

  @override
  String get walletPltAvailable => 'PLT 可用';

  @override
  String walletBalanceCount(int count) {
    return '$count 条余额';
  }

  @override
  String get shop => '商店';

  @override
  String get marketplace => '市集';

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

  @override
  String get marketplacePlaceDeal => '发布交易';

  @override
  String get marketplaceTakeDeal => '接单';

  @override
  String get marketplaceCancelDeal => '取消交易';

  @override
  String get marketplaceEmpty => '暂无开放交易。';

  @override
  String get marketplaceEligible => '可以发布交易。';

  @override
  String get marketplaceNotEligible => '未通过门槛，无法发布。';

  @override
  String marketplaceListingFee(String fee) {
    return '上架费：$fee PLT';
  }

  @override
  String get marketplaceListingFeeLabel => '上架费';

  @override
  String get marketplaceGatePlt => '需购买平台币';

  @override
  String get marketplaceGateGame => '需绑定已上架游戏';

  @override
  String get marketplaceGateTransfer => '需开通合作方 Transfer API';

  @override
  String get marketplaceWant => '求购';

  @override
  String get marketplaceOffer => '出售';

  @override
  String get marketplaceGame => '游戏';

  @override
  String get marketplaceOfferType => '出售类型';

  @override
  String get marketplaceOfferGameCoin => '游戏币';

  @override
  String get marketplaceOfferCompanyCoin => '公司币';

  @override
  String get marketplaceOfferGameItem => '游戏道具';

  @override
  String get marketplaceOfferCoinId => '出售 game_coin_id';

  @override
  String get marketplaceOfferAmount => '出售数量';

  @override
  String get marketplaceWantType => '求购类型';

  @override
  String get marketplaceWantPlt => '平台币';

  @override
  String get marketplaceWantCompanyCoin => '公司币';

  @override
  String get marketplaceWantCoinId => '求购 game_coin_id';

  @override
  String get marketplaceWantAmount => '求购数量（价格）';

  @override
  String get marketplaceItemRef => '道具';

  @override
  String get marketplacePlaced => '交易已发布';

  @override
  String get marketplaceTaken => '已接单并结算';

  @override
  String get marketplaceTakeFailed => '接单失败';

  @override
  String get marketplaceCancelled => '交易已取消';

  @override
  String get marketplaceDealNotFound => '找不到交易';

  @override
  String marketplaceDealTitle(int id) {
    return '交易 #$id';
  }

  @override
  String get marketplaceYourGameAccount => '你的游戏账号';

  @override
  String get marketplaceBack => '返回';

  @override
  String get corpRegisterTitle => '公司注册';

  @override
  String get corpLoginTitle => '公司登录';

  @override
  String get corpHomeTitle => '公司门户';

  @override
  String get corpRegisterHint => '创建公司账号并上传商业登记证（BR）。管理员批准后才能提交游戏。';

  @override
  String get corpCompanyName => '公司名称';

  @override
  String get corpContactName => '联系人';

  @override
  String get corpContactEmail => '联系邮箱';

  @override
  String get corpUploadBr => '上传商业登记证（BR）';

  @override
  String get corpBrSelected => 'BR 文件';

  @override
  String get corpBrRequired => '必须上传 BR 文件';

  @override
  String get corpSubmitRegistration => '提交审核';

  @override
  String get corpRegisterSubmitted => '已提交注册，等待管理员审核';

  @override
  String get corpHaveAccount => '已有账号？登录';

  @override
  String get corpNoAccount => '新公司？注册';

  @override
  String get corpLoginField => '邮箱或 Master Code';

  @override
  String get corpNeedLogin => '请先登录';

  @override
  String get corpMasterCode => 'Master Code';

  @override
  String corpSecretOnce(String secret) {
    return 'API Secret（仅显示一次）：$secret';
  }

  @override
  String get corpPendingApproval => 'BR 审核中。管理员批准后即可提交游戏。';

  @override
  String get corpRejected => '注册已被拒绝';

  @override
  String get corpSubmitGame => '提交游戏';

  @override
  String get corpGames => '我的游戏';

  @override
  String get corpNoGames => '尚未提交游戏。';

  @override
  String get corpGameWizardHint => '提交游戏资料及政府批文/核准材料。状态为待审直到管理员确认。';

  @override
  String get corpGameName => '游戏名称';

  @override
  String get corpGameCode => '游戏代码';

  @override
  String get corpPartnerCode => 'Partner code（可选）';

  @override
  String get corpPartnerGameId => 'Partner game id（可选）';

  @override
  String get corpGameDescription => '说明';

  @override
  String get corpUploadGovDoc => '上传政府批文';

  @override
  String get corpUploadMaterial => '上传核准材料';

  @override
  String get corpGovSelected => '政府文件';

  @override
  String get corpMaterialSelected => '核准材料';

  @override
  String get corpGameDocRequired => '至少上传一份文件';

  @override
  String get corpGameSubmitted => '游戏已提交审核';
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
  String get wallet => '钱包';

  @override
  String get walletProfile => '资料';

  @override
  String get walletEmail => '邮箱';

  @override
  String get walletHint => '商店结算或充值/提现回调后余额会更新。请刷新重新加载。';

  @override
  String get walletBalances => '余额';

  @override
  String get walletTransactions => '交易记录';

  @override
  String get walletEmptyBalances => '暂无钱包余额。';

  @override
  String get walletEmptyTxns => '暂无资金交易。';

  @override
  String get walletAvailable => '可用';

  @override
  String get walletLocked => '锁定';

  @override
  String get walletGameCoin => '游戏币';

  @override
  String get walletAmount => '数量';

  @override
  String get walletDeposit => '充值';

  @override
  String get walletWithdraw => '提现';

  @override
  String get walletTxnId => '单号';

  @override
  String get walletTxnType => '类型';

  @override
  String get walletPltAvailable => 'PLT 可用';

  @override
  String walletBalanceCount(int count) {
    return '$count 条余额';
  }

  @override
  String get shop => '商店';

  @override
  String get marketplace => '市集';

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

  @override
  String get marketplacePlaceDeal => '发布交易';

  @override
  String get marketplaceTakeDeal => '接单';

  @override
  String get marketplaceCancelDeal => '取消交易';

  @override
  String get marketplaceEmpty => '暂无开放交易。';

  @override
  String get marketplaceEligible => '可以发布交易。';

  @override
  String get marketplaceNotEligible => '未通过门槛，无法发布。';

  @override
  String marketplaceListingFee(String fee) {
    return '上架费：$fee PLT';
  }

  @override
  String get marketplaceListingFeeLabel => '上架费';

  @override
  String get marketplaceGatePlt => '需购买平台币';

  @override
  String get marketplaceGateGame => '需绑定已上架游戏';

  @override
  String get marketplaceGateTransfer => '需开通合作方 Transfer API';

  @override
  String get marketplaceWant => '求购';

  @override
  String get marketplaceOffer => '出售';

  @override
  String get marketplaceGame => '游戏';

  @override
  String get marketplaceOfferType => '出售类型';

  @override
  String get marketplaceOfferGameCoin => '游戏币';

  @override
  String get marketplaceOfferCompanyCoin => '公司币';

  @override
  String get marketplaceOfferGameItem => '游戏道具';

  @override
  String get marketplaceOfferCoinId => '出售 game_coin_id';

  @override
  String get marketplaceOfferAmount => '出售数量';

  @override
  String get marketplaceWantType => '求购类型';

  @override
  String get marketplaceWantPlt => '平台币';

  @override
  String get marketplaceWantCompanyCoin => '公司币';

  @override
  String get marketplaceWantCoinId => '求购 game_coin_id';

  @override
  String get marketplaceWantAmount => '求购数量（价格）';

  @override
  String get marketplaceItemRef => '道具';

  @override
  String get marketplacePlaced => '交易已发布';

  @override
  String get marketplaceTaken => '已接单并结算';

  @override
  String get marketplaceTakeFailed => '接单失败';

  @override
  String get marketplaceCancelled => '交易已取消';

  @override
  String get marketplaceDealNotFound => '找不到交易';

  @override
  String marketplaceDealTitle(int id) {
    return '交易 #$id';
  }

  @override
  String get marketplaceYourGameAccount => '你的游戏账号';

  @override
  String get marketplaceBack => '返回';

  @override
  String get corpRegisterTitle => '公司注册';

  @override
  String get corpLoginTitle => '公司登录';

  @override
  String get corpHomeTitle => '公司门户';

  @override
  String get corpRegisterHint => '创建公司账号并上传商业登记证（BR）。管理员批准后才能提交游戏。';

  @override
  String get corpCompanyName => '公司名称';

  @override
  String get corpContactName => '联系人';

  @override
  String get corpContactEmail => '联系邮箱';

  @override
  String get corpUploadBr => '上传商业登记证（BR）';

  @override
  String get corpBrSelected => 'BR 文件';

  @override
  String get corpBrRequired => '必须上传 BR 文件';

  @override
  String get corpSubmitRegistration => '提交审核';

  @override
  String get corpRegisterSubmitted => '已提交注册，等待管理员审核';

  @override
  String get corpHaveAccount => '已有账号？登录';

  @override
  String get corpNoAccount => '新公司？注册';

  @override
  String get corpLoginField => '邮箱或 Master Code';

  @override
  String get corpNeedLogin => '请先登录';

  @override
  String get corpMasterCode => 'Master Code';

  @override
  String corpSecretOnce(String secret) {
    return 'API Secret（仅显示一次）：$secret';
  }

  @override
  String get corpPendingApproval => 'BR 审核中。管理员批准后即可提交游戏。';

  @override
  String get corpRejected => '注册已被拒绝';

  @override
  String get corpSubmitGame => '提交游戏';

  @override
  String get corpGames => '我的游戏';

  @override
  String get corpNoGames => '尚未提交游戏。';

  @override
  String get corpGameWizardHint => '提交游戏资料及政府批文/核准材料。状态为待审直到管理员确认。';

  @override
  String get corpGameName => '游戏名称';

  @override
  String get corpGameCode => '游戏代码';

  @override
  String get corpPartnerCode => 'Partner code（可选）';

  @override
  String get corpPartnerGameId => 'Partner game id（可选）';

  @override
  String get corpGameDescription => '说明';

  @override
  String get corpUploadGovDoc => '上传政府批文';

  @override
  String get corpUploadMaterial => '上传核准材料';

  @override
  String get corpGovSelected => '政府文件';

  @override
  String get corpMaterialSelected => '核准材料';

  @override
  String get corpGameDocRequired => '至少上传一份文件';

  @override
  String get corpGameSubmitted => '游戏已提交审核';
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
  String get wallet => '錢包';

  @override
  String get walletProfile => '資料';

  @override
  String get walletEmail => '電子郵件';

  @override
  String get walletHint => '商店結算或儲值/提現回調後餘額會更新。請重新整理。';

  @override
  String get walletBalances => '餘額';

  @override
  String get walletTransactions => '交易紀錄';

  @override
  String get walletEmptyBalances => '尚無錢包餘額。';

  @override
  String get walletEmptyTxns => '尚無資金交易。';

  @override
  String get walletAvailable => '可用';

  @override
  String get walletLocked => '鎖定';

  @override
  String get walletGameCoin => '遊戲幣';

  @override
  String get walletAmount => '數量';

  @override
  String get walletDeposit => '儲值';

  @override
  String get walletWithdraw => '提現';

  @override
  String get walletTxnId => '單號';

  @override
  String get walletTxnType => '類型';

  @override
  String get walletPltAvailable => 'PLT 可用';

  @override
  String walletBalanceCount(int count) {
    return '$count 筆餘額';
  }

  @override
  String get shop => '商店';

  @override
  String get marketplace => '市集';

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

  @override
  String get marketplacePlaceDeal => '發布交易';

  @override
  String get marketplaceTakeDeal => '接單';

  @override
  String get marketplaceCancelDeal => '取消交易';

  @override
  String get marketplaceEmpty => '暫無開放交易。';

  @override
  String get marketplaceEligible => '可以發布交易。';

  @override
  String get marketplaceNotEligible => '未通過門檻，無法發布。';

  @override
  String marketplaceListingFee(String fee) {
    return '上架費：$fee PLT';
  }

  @override
  String get marketplaceListingFeeLabel => '上架費';

  @override
  String get marketplaceGatePlt => '需購買平台幣';

  @override
  String get marketplaceGateGame => '需綁定已上架遊戲';

  @override
  String get marketplaceGateTransfer => '需開通合作方 Transfer API';

  @override
  String get marketplaceWant => '求購';

  @override
  String get marketplaceOffer => '出售';

  @override
  String get marketplaceGame => '遊戲';

  @override
  String get marketplaceOfferType => '出售類型';

  @override
  String get marketplaceOfferGameCoin => '遊戲幣';

  @override
  String get marketplaceOfferCompanyCoin => '公司幣';

  @override
  String get marketplaceOfferGameItem => '遊戲道具';

  @override
  String get marketplaceOfferCoinId => '出售 game_coin_id';

  @override
  String get marketplaceOfferAmount => '出售數量';

  @override
  String get marketplaceWantType => '求購類型';

  @override
  String get marketplaceWantPlt => '平台幣';

  @override
  String get marketplaceWantCompanyCoin => '公司幣';

  @override
  String get marketplaceWantCoinId => '求購 game_coin_id';

  @override
  String get marketplaceWantAmount => '求購數量（價格）';

  @override
  String get marketplaceItemRef => '道具';

  @override
  String get marketplacePlaced => '交易已發布';

  @override
  String get marketplaceTaken => '已接單並結算';

  @override
  String get marketplaceTakeFailed => '接單失敗';

  @override
  String get marketplaceCancelled => '交易已取消';

  @override
  String get marketplaceDealNotFound => '找不到交易';

  @override
  String marketplaceDealTitle(int id) {
    return '交易 #$id';
  }

  @override
  String get marketplaceYourGameAccount => '你的遊戲帳號';

  @override
  String get marketplaceBack => '返回';

  @override
  String get corpRegisterTitle => '公司註冊';

  @override
  String get corpLoginTitle => '公司登入';

  @override
  String get corpHomeTitle => '公司入口';

  @override
  String get corpRegisterHint => '建立公司帳號並上傳商業登記證（BR）。管理員核准後才能提交遊戲。';

  @override
  String get corpCompanyName => '公司名稱';

  @override
  String get corpContactName => '聯絡人';

  @override
  String get corpContactEmail => '聯絡信箱';

  @override
  String get corpUploadBr => '上傳商業登記證（BR）';

  @override
  String get corpBrSelected => 'BR 檔案';

  @override
  String get corpBrRequired => '必須上傳 BR 檔案';

  @override
  String get corpSubmitRegistration => '提交審核';

  @override
  String get corpRegisterSubmitted => '已提交註冊，等待管理員審核';

  @override
  String get corpHaveAccount => '已有帳號？登入';

  @override
  String get corpNoAccount => '新公司？註冊';

  @override
  String get corpLoginField => '信箱或 Master Code';

  @override
  String get corpNeedLogin => '請先登入';

  @override
  String get corpMasterCode => 'Master Code';

  @override
  String corpSecretOnce(String secret) {
    return 'API Secret（僅顯示一次）：$secret';
  }

  @override
  String get corpPendingApproval => 'BR 審核中。管理員核准後即可提交遊戲。';

  @override
  String get corpRejected => '註冊已被拒絕';

  @override
  String get corpSubmitGame => '提交遊戲';

  @override
  String get corpGames => '我的遊戲';

  @override
  String get corpNoGames => '尚未提交遊戲。';

  @override
  String get corpGameWizardHint => '提交遊戲資料及政府批文/核准材料。狀態為待審直到管理員確認。';

  @override
  String get corpGameName => '遊戲名稱';

  @override
  String get corpGameCode => '遊戲代碼';

  @override
  String get corpPartnerCode => 'Partner code（選填）';

  @override
  String get corpPartnerGameId => 'Partner game id（選填）';

  @override
  String get corpGameDescription => '說明';

  @override
  String get corpUploadGovDoc => '上傳政府批文';

  @override
  String get corpUploadMaterial => '上傳核准材料';

  @override
  String get corpGovSelected => '政府文件';

  @override
  String get corpMaterialSelected => '核准材料';

  @override
  String get corpGameDocRequired => '至少上傳一份文件';

  @override
  String get corpGameSubmitted => '遊戲已提交審核';
}
