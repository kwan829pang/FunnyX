// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get appTitle => 'FunnyX 管理后台';

  @override
  String get login => '登录';

  @override
  String get username => '账号';

  @override
  String get password => '密码';

  @override
  String get dashboard => '仪表盘';

  @override
  String get markets => '市场';

  @override
  String get packages => 'PLT 套餐';

  @override
  String get corpProducts => '企业商品';

  @override
  String get setup => '系统初始化';

  @override
  String get logout => '登出';

  @override
  String get cancel => '取消';

  @override
  String get status => '状态';

  @override
  String get actions => '操作';

  @override
  String get commandPalette => '搜索';

  @override
  String get loginHint => '公开登录。其他页面需要会话令牌。';

  @override
  String get setupWizardTitle => '系统初始化向导';

  @override
  String get setupWizardSubtitle => '使用管理后台前，请先配置平台必需变量。';

  @override
  String get setupStepWelcome => '必需变量';

  @override
  String get setupStepBaseFiat => '基础法币';

  @override
  String get setupStepPackages => 'PLT 套餐价格';

  @override
  String get setupStepConfirm => '确认';

  @override
  String get setupWelcomeBody =>
      'FunnyX 需要基础法币（HKD 或 USD），以及至少一个已定价的平台 PLT 套餐，用于电子商店与企业费用。';

  @override
  String get setupBaseFiatHint => '该币种用于全部电子商店标价与 C6 企业费用账单。';

  @override
  String get setupPackagesHint => '为固定平台代币套餐（PLT_*）设置法币价格，至少一个需保持启用。';

  @override
  String get setupConfirmHint => '请核对以下配置，完成后将标记系统已初始化。';

  @override
  String get setupFiatPrice => '法币价格';

  @override
  String get setupNext => '下一步';

  @override
  String get setupSaveContinue => '保存并继续';

  @override
  String get setupFinish => '完成初始化';

  @override
  String get packagesHint => '创建并管理固定平台代币套餐，代码必须以 PLT_ 开头。';

  @override
  String get packagesEmpty => '尚无平台套餐。';

  @override
  String get packagesCreate => '创建套餐';

  @override
  String get packagesCode => '代码';

  @override
  String get packagesName => '名称';

  @override
  String get packagesCoinAmount => '代币数量';

  @override
  String get corpProductsHint => '监控企业电子商店商品，可用停用或归档进行管控。';

  @override
  String get corpProductsEmpty => '未找到企业商品。';

  @override
  String get corpId => '企业 ID';

  @override
  String get productType => '类型';

  @override
  String get statusFilterAll => '全部状态';

  @override
  String get statusDraft => '草稿';

  @override
  String get statusActive => '启用';

  @override
  String get statusInactive => '停用';

  @override
  String get statusArchived => '归档';
}

/// The translations for Chinese, as used in China (`zh_CN`).
class AppLocalizationsZhCn extends AppLocalizationsZh {
  AppLocalizationsZhCn() : super('zh_CN');

  @override
  String get appTitle => 'FunnyX 管理后台';

  @override
  String get login => '登录';

  @override
  String get username => '账号';

  @override
  String get password => '密码';

  @override
  String get dashboard => '仪表盘';

  @override
  String get markets => '市场';

  @override
  String get logout => '登出';

  @override
  String get commandPalette => '搜索';

  @override
  String get loginHint => '公开登录。其他页面需要会话令牌。';
}

/// The translations for Chinese, as used in Taiwan (`zh_TW`).
class AppLocalizationsZhTw extends AppLocalizationsZh {
  AppLocalizationsZhTw() : super('zh_TW');

  @override
  String get appTitle => 'FunnyX 管理後台';

  @override
  String get login => '登入';

  @override
  String get username => '帳號';

  @override
  String get password => '密碼';

  @override
  String get dashboard => '儀表板';

  @override
  String get markets => '市場';

  @override
  String get logout => '登出';

  @override
  String get commandPalette => '搜尋';

  @override
  String get loginHint => '公開登入。其他頁面需工作階段權杖。';
}
