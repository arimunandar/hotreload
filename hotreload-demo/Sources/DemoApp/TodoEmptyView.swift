import SwiftUI

struct TodoEmptyView: View {
    let filter: TodoFilter
    let onAdd: () -> Void

    var body: some View {
        VStack(spacing: 20) {
            // Illustration
            ZStack {
                Circle()
                    .fill(Color.purple.opacity(0.08))
                    .frame(width: 100, height: 100)

                Circle()
                    .fill(Color.purple.opacity(0.12))
                    .frame(width: 72, height: 72)

                Image(systemName: icon)
                    .font(.system(size: 32))
                    .foregroundStyle(.purple)
            }

            VStack(spacing: 6) {
                Text(message)
                    .font(.title3.weight(.semibold))
                    .foregroundStyle(.primary)

                Text(subtitle)
                    .font(.subheadline)
                    .foregroundStyle(.tertiary)
                    .multilineTextAlignment(.center)
            }
            .padding(.horizontal, 32)

            if filter == .all {
                Button(action: onAdd) {
                    Label("Add Your First Task", systemImage: "plus")
                        .font(.subheadline.weight(.medium))
                        .padding(.horizontal, 20)
                        .padding(.vertical, 10)
                        .background(Color.purple)
                        .foregroundStyle(.white)
                        .clipShape(Capsule())
                }
            }
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 48)
        .background(Color(.systemBackground))
        .clipShape(RoundedRectangle(cornerRadius: 20))
        .shadow(color: .black.opacity(0.04), radius: 8, y: 2)
    }

    private var icon: String {
        switch filter {
        case .all:      return "tray"
        case .active:   return "checkmark.circle"
        case .completed: return "star"
        }
    }

    private var message: String {
        switch filter {
        case .all:      return "Nothing here yet"
        case .active:   return "All caught up!"
        case .completed: return "No completed tasks"
        }
    }

    private var subtitle: String {
        switch filter {
        case .all:      return "Tap the button below to create your first task and get started"
        case .active:   return "You've completed every active task — great work!"
        case .completed: return "Complete a task and it will show up here"
        }
    }
}
