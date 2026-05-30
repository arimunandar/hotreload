import SwiftUI

struct GreetingBanner: View {
    @ObservedObject var store: TodoStore

    @State private var animateGradient = false

    private var greeting: String {
        let hour = Calendar.current.component(.hour, from: Date())
        switch hour {
        case 5..<12:  return "Good Morning"
        case 12..<17: return "Good Afternoon"
        case 17..<22: return "Good Evening"
        default:      return "Good Night"
        }
    }

    var body: some View {
        HStack(spacing: 16) {
            // Avatar
            ZStack {
                Circle()
                    .fill(.white.opacity(0.25))
                    .frame(width: 52, height: 52)

                Image(systemName: "person.circle.fill")
                    .font(.system(size: 44))
                    .foregroundStyle(.white.opacity(0.9))
            }

            VStack(alignment: .leading, spacing: 4) {
                Text(greeting)
                    .font(.subheadline.weight(.medium))
                    .foregroundStyle(.white.opacity(0.8))

                Text("Ari")
                    .font(.title2.weight(.bold))
                    .foregroundStyle(.white)

                HStack(spacing: 6) {
                    Circle()
                        .fill(.green)
                        .frame(width: 6, height: 6)
                    Text("\(store.activeCount) active tasks")
                        .font(.caption.weight(.medium))
                        .foregroundStyle(.white.opacity(0.85))
                }
            }

            Spacer()

            // Decorative date badge
            VStack(spacing: 2) {
                Text(Date.now, format: .dateTime.day())
                    .font(.title3.weight(.bold))
                    .foregroundStyle(.white)
                Text(Date.now, format: .dateTime.month(.abbreviated))
                    .font(.caption.weight(.medium))
                    .foregroundStyle(.white.opacity(0.8))
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 12))
        }
        .padding(16)
        .background(
            ZStack {
                LinearGradient(
                    colors: [.purple, .blue, .teal],
                    startPoint: animateGradient ? .topLeading : .bottomLeading,
                    endPoint: animateGradient ? .bottomTrailing : .topTrailing
                )

                // Decorative orbs
                Circle()
                    .fill(.white.opacity(0.08))
                    .frame(width: 120)
                    .offset(x: -20, y: -40)

                Circle()
                    .fill(.white.opacity(0.05))
                    .frame(width: 80)
                    .offset(x: 40, y: 30)
            }
        )
        .clipShape(RoundedRectangle(cornerRadius: 20))
        .shadow(color: .blue.opacity(0.2), radius: 15, y: 5)
        .onAppear {
            withAnimation(.linear(duration: 4).repeatForever(autoreverses: true)) {
                animateGradient.toggle()
            }
        }
    }
}
