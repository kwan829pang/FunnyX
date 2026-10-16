import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/corp_onboard_controller.dart';
import '../../widgets/page_frame.dart';

class CorpOnboardPage extends StatefulWidget {
  const CorpOnboardPage({super.key});

  @override
  State<CorpOnboardPage> createState() => _CorpOnboardPageState();
}

class _CorpOnboardPageState extends State<CorpOnboardPage>
    with SingleTickerProviderStateMixin {
  late final TabController _tabs;

  @override
  void initState() {
    super.initState();
    _tabs = TabController(length: 2, vsync: this);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      Get.find<CorpOnboardController>().load();
    });
  }

  @override
  void dispose() {
    _tabs.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ctrl = Get.find<CorpOnboardController>();
    return PageFrame(
      title: 'Corp onboarding',
      subtitle: 'Approve or reject pending companies (BR) and games.',
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          TabBar(
            controller: _tabs,
            tabs: const [
              Tab(text: 'Companies'),
              Tab(text: 'Games'),
            ],
          ),
          Expanded(
            child: Obx(() {
              if (ctrl.loading.value && ctrl.corps.isEmpty) {
                return const Center(child: CircularProgressIndicator());
              }
              if (ctrl.error.value != null && ctrl.corps.isEmpty) {
                return Center(child: Text(ctrl.error.value!));
              }
              return TabBarView(
                controller: _tabs,
                children: [
                  _corpList(ctrl),
                  _gameList(ctrl),
                ],
              );
            }),
          ),
        ],
      ),
    );
  }

  Widget _corpList(CorpOnboardController ctrl) {
    if (ctrl.corps.isEmpty) {
      return const Center(child: Text('No corporate users'));
    }
    return ListView.separated(
      itemCount: ctrl.corps.length,
      separatorBuilder: (_, _) => const Divider(height: 1),
      itemBuilder: (context, i) {
        final c = ctrl.corps[i];
        final id = (c['id'] as num?)?.toInt() ?? 0;
        final status = c['status']?.toString() ?? '';
        return ListTile(
          title: Text(c['company_name']?.toString() ?? ''),
          subtitle: Text(
            '${c['contact_email'] ?? ''} · $status · docs=${(c['documents'] as List?)?.length ?? 0}',
          ),
          trailing: status == 'pending'
              ? Wrap(
                  spacing: 8,
                  children: [
                    TextButton(
                      onPressed: () => ctrl.approveCorp(id),
                      child: const Text('Approve'),
                    ),
                    TextButton(
                      onPressed: () => ctrl.rejectCorp(id),
                      child: const Text('Reject'),
                    ),
                  ],
                )
              : Text(status),
        );
      },
    );
  }

  Widget _gameList(CorpOnboardController ctrl) {
    if (ctrl.games.isEmpty) {
      return const Center(child: Text('No games'));
    }
    return ListView.separated(
      itemCount: ctrl.games.length,
      separatorBuilder: (_, _) => const Divider(height: 1),
      itemBuilder: (context, i) {
        final g = ctrl.games[i];
        final id = (g['id'] as num?)?.toInt() ?? 0;
        final status = g['status']?.toString() ?? '';
        return ListTile(
          title: Text('${g['game_name']} (${g['game_code']})'),
          subtitle: Text('${g['company_name']} · $status'),
          trailing: status == 'pending'
              ? Wrap(
                  spacing: 8,
                  children: [
                    TextButton(
                      onPressed: () => ctrl.approveGame(id),
                      child: const Text('Approve'),
                    ),
                    TextButton(
                      onPressed: () => ctrl.rejectGame(id),
                      child: const Text('Reject'),
                    ),
                  ],
                )
              : Text(status),
        );
      },
    );
  }
}
