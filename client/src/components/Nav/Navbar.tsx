import Link from "next/link"
import nav from "./nav.module.css"
import Image from "next/image"

const Nav = () => {
	return (
		<div className={nav.page}>
			<div
				className="logo"
				style={{
					display: "flex",
					flexDirection: "row",
					alignItems: "center",
					width: "50vw",
				}}
			>
				<Image src="/logo.svg" alt="logo" width="50" height="50" />
				<span
					style={{
						marginLeft: "20px",
						fontSize: "20px",
						letterSpacing: "-1px",
					}}
				>
					Wares
				</span>
			</div>
			<div
				style={{
					width: "30vw",
					display: "flex",
					justifyContent: "space-between",
				}}
			>
				<Link href="/teams">Teams</Link>
				<Link href="/inbound">Inbound</Link>
				<Link href="/fulfillment">Fulfillment</Link>
			</div>
		</div>
	)
}

export default Nav
