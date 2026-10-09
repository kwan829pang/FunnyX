// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'FunnyX Admin';

  @override
  String get login => 'Login';

  @override
  String get username => 'Username';

  @override
  String get password => 'Password';

  @override
  String get dashboard => 'Dashboard';

  @override
  String get markets => 'Markets';

  @override
  String get packages => 'PLT packages';

  @override
  String get corpProducts => 'Corp products';

  @override
  String get setup => 'System setup';

  @override
  String get logout => 'Logout';

  @override
  String get cancel => 'Cancel';

  @override
  String get status => 'Status';

  @override
  String get actions => 'Actions';

  @override
  String get commandPalette => 'Search';

  @override
  String get loginHint => 'Public login. Other pages require a session token.';

  @override
  String get setupWizardTitle => 'System setup wizard';

  @override
  String get setupWizardSubtitle =>
      'Configure required platform variables before using Admin Panel.';

  @override
  String get setupStepWelcome => 'Required variables';

  @override
  String get setupStepBaseFiat => 'Base fiat currency';

  @override
  String get setupStepPackages => 'PLT package prices';

  @override
  String get setupStepConfirm => 'Confirm';

  @override
  String get setupWelcomeBody =>
      'FunnyX needs a base fiat currency (HKD or USD) and at least one priced platform PLT package for the e-shop and Corp fees.';

  @override
  String get setupBaseFiatHint =>
      'This currency applies to all e-shop list prices and C6 Corp fee invoices.';

  @override
  String get setupPackagesHint =>
      'Set fiat prices for fixed Platform Token packages (PLT_*). At least one must stay active.';

  @override
  String get setupConfirmHint =>
      'Review the values below, then finish to mark the system as initialized.';

  @override
  String get setupFiatPrice => 'Fiat price';

  @override
  String get setupNext => 'Next';

  @override
  String get setupSaveContinue => 'Save and continue';

  @override
  String get setupFinish => 'Finish setup';

  @override
  String get packagesHint =>
      'Seed and manage fixed Platform Token packages. Codes must start with PLT_.';

  @override
  String get packagesEmpty => 'No platform packages yet.';

  @override
  String get packagesCreate => 'Create package';

  @override
  String get packagesCode => 'Code';

  @override
  String get packagesName => 'Name';

  @override
  String get packagesCoinAmount => 'Coin amount';

  @override
  String get corpProductsHint =>
      'Monitor Corp e-shop products. Suspend with inactive or archive.';

  @override
  String get corpProductsEmpty => 'No Corp products found.';

  @override
  String get corpId => 'Corp ID';

  @override
  String get productType => 'Type';

  @override
  String get statusFilterAll => 'All statuses';

  @override
  String get statusDraft => 'Draft';

  @override
  String get statusActive => 'Active';

  @override
  String get statusInactive => 'Inactive';

  @override
  String get statusArchived => 'Archived';
}
